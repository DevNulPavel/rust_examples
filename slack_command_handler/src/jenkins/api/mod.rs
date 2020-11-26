// mod job_info_json;
mod job_info_xml;
mod jobs_list;

pub use self::{
    // job_info_json::{
    //     JenkinsJobParameter,
    //     JenkinsJobParameterDefaultBoolValue,
    //     JenkinsJobParameterDefaultStringValue,
    //     InfoRequestError,
    //     request_jenkins_job_info
    // },
    job_info_xml::{
        Parameter,
        ChoiseInfo,
        ChoiseList,
        InfoRequestError,
        request_jenkins_job_info
    },
    jobs_list::{
        request_jenkins_jobs_list,
        JenkinsJob
    }
};