use lazy_static::{
    lazy_static
};
use regex::{
    Regex
};


#[derive(Debug)]
pub struct MentionParseResult<'a>{
    pub bot_id: &'a str,
    pub target_name: &'a str,
    pub branch_name: &'a str
}

pub fn parse_mention_params<'a>(text: &'a str) -> Option<MentionParseResult<'a>>{
    lazy_static! {
        static ref BUILD_MENTION: Regex = Regex::new(
            r"^\s*<@(?P<bot_id>[A-Z0-9]+)>\s+(?P<target_name>[A-Za-z0-9_-]+)\s+(?P<branch>[A-Za-z0-9_-]+)\s*$"
        ).unwrap();
    }    

    let groups = BUILD_MENTION
        .captures(text)?;

    let bot_id = groups
        .name("bot_id")
        .map(|val| val.as_str())?;

    let target_name = groups
        .name("target_name")
        .map(|val| val.as_str())?;

    let branch_name = groups
        .name("branch")
        .map(|val| val.as_str())?;
    
    Some(MentionParseResult{
        bot_id,
        target_name,
        branch_name
    })
}


#[cfg(test)]
mod tests{
    use super::{
        parse_mention_params
    };

    #[test]
    fn test_mention_regex_expression(){
        {
            let params = parse_mention_params(" <@UASD123> pi2   pi2_tasks_test")
                .expect("Build mention test failed 1");

            assert_eq!(params.bot_id, "UASD123");
            assert_eq!(params.target_name, "pi2");
            assert_eq!(params.branch_name, "pi2_tasks_test");
        }

        {
            let params = parse_mention_params("<@UASD123> pi2   pi2_tasks_test  ")
                .expect("Build mention test failed 1");

            assert_eq!(params.bot_id, "UASD123");
            assert_eq!(params.target_name, "pi2");
            assert_eq!(params.branch_name, "pi2_tasks_test");
        }

        {
            assert_eq!(
                parse_mention_params("This is the test text <@UASD123> pi2 pi2_tasks_test").is_none(), 
                true
            );
        }
    }
}