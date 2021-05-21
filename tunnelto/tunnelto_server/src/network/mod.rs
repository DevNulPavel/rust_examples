mod server;
mod proxy;

use std::{
    net::{
        IpAddr, 
        SocketAddr
    }
};
use futures::{
    future::{
        select_ok
    },
    FutureExt
};
use thiserror::{
    Error
};
pub use self::{
    server::{
        spawn
    },
    proxy::{
        proxy_stream
    }
};
use reqwest::{
    StatusCode
};
use trust_dns_resolver::{
    TokioAsyncResolver
};
use crate::{
    network::{
        server::{
            HostQuery, 
            HostQueryResponse
        }
    },
    ClientId
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("IOError: {0}")]
    IoError(#[from] std::io::Error),

    #[error("RequestError: {0}")]
    Request(#[from] reqwest::Error),

    #[error("ResolverError: {0}")]
    Resolver(#[from] trust_dns_resolver::error::ResolveError),

    #[error("Does not serve host")]
    DoesNotServeHost,
}

/// Экземпляр нашего внутреннего сервера
#[derive(Debug, Clone)]
pub struct Instance {
    pub ip: IpAddr,
}

impl Instance {
    /// Получаем все инстансы, где наше приложение-сервер запущено
    async fn get_instances() -> Result<Vec<Instance>, Error> {
        // Получаем DNS адрес поиска серверов
        let query = if let Some(dns) = crate::CONFIG.gossip_dns_host.clone() {
            dns
        } else {
            tracing::warn!("warning! gossip mode disabled!");
            return Ok(vec![]);
        };

        tracing::debug!("querying app instances");

        // Создаем резолвер DNS адресов
        let resolver = TokioAsyncResolver::tokio_from_system_conf()?;

        // Находим все адреса DNS данного адреса
        let ips = resolver
            .lookup_ip(query)
            .await?;

        // Полученный список адресов и есть наши сервера
        let instances = ips
            .iter()
            .map(|ip| Instance { ip })
            .collect();

        tracing::debug!("Found app instances: {:?}", &instances);
        Ok(instances)
    }

    /// Запрашиваем инстанс и смотрим, запускает ли он наш хост??
    async fn serves_host(self, host: &str) -> Result<(Instance, ClientId), Error> {
        // Адрес текущего внутреннего коммуникационного сервера и порт
        let addr = SocketAddr::new(self.ip.clone(), 
                                   crate::CONFIG.internal_network_port);

        // Адрес в виде строки                                   
        let url = format!("http://{}", addr.to_string());

        // Делаем запрос на корень нашего внутреннего сервера
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .query(&HostQuery {
                host: host.to_string(),
            })
            .send()
            .await?;

        // Статус и ответ от нашего сервера
        let status = response.status();
        let result: HostQueryResponse = response
            .json()
            .await?;

        // Идентификатор клиента, пустой если нету
        let found_client = result
            .client_id
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default();
        tracing::debug!(status=%status, found=%found_client, "got net svc response");

        match (status, result.client_id) {
            (StatusCode::OK, Some(client_id)) => Ok((self, client_id)),
            _ => Err(Error::DoesNotServeHost),
        }
    }
}

/// Получаем для хоста инстанс и идентификатор
#[tracing::instrument]
pub async fn instance_for_host(host: &str) -> Result<(Instance, ClientId), Error> {
    // Итератор по футурам для получения инстанса
    let instances = Instance::get_instances()
        .await?
        .into_iter()
        .map(|i| {
            // Обслуживает ли инстанс конкретный хост?
            i.serves_host(host).boxed()
        });

    if instances.len() == 0 {
        return Err(Error::DoesNotServeHost);
    }

    // Получаем первый успешный ответ
    let instance = select_ok(instances)
        .await?
        .0;

    tracing::info!(instance_ip = %instance.0.ip, 
                   client_id = %instance.1.to_string(), 
                   subdomain=%host, 
                   "found instance for host");
    Ok(instance)
}
