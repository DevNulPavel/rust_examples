use serde::{
    Deserialize
};

#[derive(Deserialize, Debug)]
pub struct ChoiseList{
    #[serde(rename = "string")]
    values: Vec<String>
}

#[derive(Deserialize, Debug)]
pub struct ChoiseInfo{
    #[serde(rename = "defaultChoice")]
    default_value: String,

    #[serde(rename = "choiceList")]
    choice_list: ChoiseList
}

// https://serde.rs/enum-representations.html
#[derive(Deserialize, Debug)]
pub enum Parameter{
    #[serde(rename = "hudson.model.BooleanParameterDefinition")]
    Boolean{
        name: String,
        description: String,
    
        #[serde(rename = "defaultValue")]
        default_value: bool
    },
    #[serde(rename = "hudson.model.StringParameterDefinition")]
    String{
        name: String,
        description: String,

        #[serde(rename = "defaultValue")]
        default_value: String
    },
    #[serde(rename = "jp.ikedam.jenkins.plugins.extensible__choice__parameter.ExtensibleChoiceParameterDefinition")]
    Choice{
        name: String,
        description: String,
        
        #[serde(rename = "choiceListProvider")]
        choise: ChoiseInfo,
    },
    #[serde(rename = "hudson.model.ChoiceParameterDefinition")]
    ChoiceSimple{
        name: String,
        description: String,
        
        // #[serde(rename = "choiceListProvider")]
        // choise: Choise,
    },
    #[serde(rename = "net.uaznia.lukanus.hudson.plugins.gitparameter.GitParameterDefinition")]
    Git{
        name: String,
        description: String,

        #[serde(rename = "defaultValue")]
        default_value: String
    },
    #[serde(rename = "org.jvnet.jenkins.plugins.nodelabelparameter.LabelParameterDefinition")]
    Labels{
        name: String,
        description: String,

        #[serde(rename = "defaultValue")]
        default_value: String
    },

    #[serde(other)]
    Unknown
}