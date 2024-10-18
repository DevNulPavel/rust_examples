use pyo3::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing_subscriber::EnvFilter;

#[pyclass]
struct PyTcpStream {
    inner: Option<TcpStream>,
}

#[pymethods]
impl PyTcpStream {
    fn send<'py>(&'py mut self, py: Python<'py>, content: Vec<u8>) -> PyResult<&PyAny> {
        let mut stream = self.inner.take().unwrap();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let ( r,mut w) = stream.split();
            w.write_all(&content).await.unwrap();

            // This future could hang about 900s on linux with `net.ipv4.tcp_retries2 = 15`
            let fut = r.readable();

            fut.await.unwrap();

            // Wrap into timeout could prevent this case.
            // tokio::time::timeout(std::time::Duration::from_secs(10), fut).await.unwrap().unwrap();

            Ok(Python::with_gil(|py| py.None()))
        })
    }
}

#[pyfunction]
fn connect(py: Python<'_>, address: String, port: u32) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let stream = TcpStream::connect(format!("{}:{}", address, port))
            .await
            .unwrap();
        Ok(PyTcpStream {
            inner: Some(stream),
        })
    })
}

#[pymodule]
fn tcptest(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .pretty()
        .init();

    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add_class::<PyTcpStream>()?;
    Ok(())
}
