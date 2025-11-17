use crate::{bamboo, build_status::BuildStatus, circle_ci, jenkins, travis};
use regex::Regex;

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub builds: Vec<BuildConfig>,
}

const PREFIX_LEN: usize = "${process.env.".len();

fn substitute_variables(s: &String, replacer: fn(&str) -> Option<String>) -> String {
    let re = Regex::new(r"\$\{process\.env\.[a-zA-Z_]*\}").unwrap();
    let matches = re.find_iter(s); //.collect::<Vec<_>>();
    let mut res = String::from("");
    let mut index = 0;
    for m in matches {
        // let start = m.start();
        let buf = &s[index..m.start()];
        index = m.end();
        res.push_str(buf);
        let m_str = m.as_str();
        let var_name = &m.as_str()[PREFIX_LEN..m_str.len() - 1];
        res.push_str(&replacer(var_name).unwrap_or_else(|| String::from("")));
    }
    if index < s.len() {
        res.push_str(&s[index..s.len()]);
    }
    res
}

pub fn env_replacer(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

pub fn load_config(s: &String, replacer: fn(&str) -> Option<String>) -> Result<Config, String> {
    let sub = substitute_variables(s, replacer);
    serde_json::from_str::<Config>(&sub).map_err(|e| format!("JSON Error {:?}", e))
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase", rename_all_fields = "camelCase", tag = "tag")]
pub enum BuildConfig {
    Bamboo {
        server_url: String,
        plan: String,
        token: Option<String>,
    },
    CircleCI {
        org: String,
        repo: String,
        branch: String,
        token: Option<String>,
    },
    Travis {
        server_url: String,
        repository: String,
        branch: String,
        token: Option<String>,
    },
    Jenkins {
        server_url: String,
        plan: String,
        branch: String,
        user: Option<String>,
        token: Option<String>,
    },
}

impl BuildConfig {
    pub async fn fetch(&self) -> Result<BuildStatus, String> {
        match self {
            Self::Bamboo {
                server_url,
                plan,
                token,
            } => bamboo::fetch(server_url, plan, token).await,
            Self::CircleCI {
                org,
                repo,
                branch,
                token,
            } => circle_ci::fetch(org, repo, branch, token).await,
            Self::Travis {
                server_url,
                repository,
                branch,
                token,
            } => travis::fetch(server_url, repository, branch, token).await,
            Self::Jenkins {
                server_url,
                plan,
                branch,
                user,
                token,
            } => jenkins::fetch(server_url, plan, branch, token, user).await,
        }
    }

    pub fn get_title(&self) -> String {
        match self {
            Self::Bamboo {
                server_url: _,
                plan,
                token: _,
            } => plan.to_string(),
            Self::CircleCI {
                org,
                repo,
                branch,
                token: _,
            } => format!("{org}/{repo}/{branch}"),
            Self::Travis {
                server_url: _,
                repository,
                branch,
                token: _,
            } => format!("{repository}/{branch}"),
            Self::Jenkins {
                server_url: _,
                plan,
                branch,
                user: _,
                token: _,
            } => format!("{plan}/{branch}"),
        }
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn decode_jenkins() {
        let config = String::from(
            "{\"pollingInterval\":60000,\"builds\":[{\"tag\":\"jenkins\",\"serverUrl\":\"https://my.jenkins\",\"user\":\"${process.env.JENKINS_USER}\",\"token\":\"${process.env.JENKINS_TOKEN}\",\"plan\":\"my-plan\",\"branch\":\"main\"}]}",
        );
        let config = serde_json::from_str::<Config>(&config).unwrap();
        let expected = Config {
            builds: vec![BuildConfig::Jenkins {
                server_url: String::from("https://my.jenkins"),
                plan: String::from("my-plan"),
                branch: String::from("main"),
                user: Some(String::from("${process.env.JENKINS_USER}")),
                token: Some(String::from("${process.env.JENKINS_TOKEN}")),
            }],
        };
        assert_eq!(config, expected)
    }

    #[test]
    fn decode_config() {
        let config = String::from(
            "{\"pollingInterval\":60000,\"builds\":[{\"tag\":\"bamboo\",\"serverUrl\":\"http://my.bamboo\",\"token\":\"${process.env.BAMBOO_TOKEN}\",\"plan\":\"MY-PLAN\",\"groups\":[\"g1\"]},{\"tag\":\"circleci\",\"org\":\"vankeisb\",\"repo\":\"react-tea-cup\",\"branch\":\"master\",\"groups\":[\"g2\"]},{\"tag\":\"travis\",\"serverUrl\":\"https://my.travis\",\"repository\":\"my/repo\",\"branch\":\"develop\",\"token\":\"${process.env.TRAVIS_TOKEN}\",\"groups\":[\"g2\"]}]}",
        );
        let config = serde_json::from_str::<Config>(&config).unwrap();
        let expected = Config {
            builds: vec![
                BuildConfig::Bamboo {
                    server_url: String::from("http://my.bamboo"),
                    plan: String::from("MY-PLAN"),
                    token: Some(String::from("${process.env.BAMBOO_TOKEN}")),
                },
                BuildConfig::CircleCI {
                    org: String::from("vankeisb"),
                    repo: String::from("react-tea-cup"),
                    branch: String::from("master"),
                    token: None,
                },
                BuildConfig::Travis {
                    server_url: String::from("https://my.travis"),
                    repository: String::from("my/repo"),
                    branch: String::from("develop"),
                    token: Some(String::from("${process.env.TRAVIS_TOKEN}")),
                },
            ],
        };
        assert_eq!(config, expected)
    }

    #[test]
    fn substitute() {
        let s = String::from("foo ${process.env.YALLA}");
        let expected = String::from("foo fonk");
        assert_eq!(
            substitute_variables(&s, |_| Some(String::from("fonk"))),
            expected
        )
    }

    #[test]
    fn load() {
        fn my_replacer(s: &str) -> Option<String> {
            if s == "BAMBOO_TOKEN" {
                Some(String::from("btoken"))
            } else {
                None
            }
        }

        let config = String::from(
            "{\"pollingInterval\":60000,\"builds\":[{\"tag\":\"bamboo\",\"serverUrl\":\"http://my.bamboo\",\"token\":\"${process.env.BAMBOO_TOKEN}\",\"plan\":\"MY-PLAN\",\"groups\":[\"g1\"]},{\"tag\":\"circleci\",\"org\":\"vankeisb\",\"repo\":\"react-tea-cup\",\"branch\":\"master\",\"groups\":[\"g2\"]},{\"tag\":\"travis\",\"serverUrl\":\"https://my.travis\",\"repository\":\"my/repo\",\"branch\":\"develop\",\"token\":\"${process.env.TRAVIS_TOKEN}\",\"groups\":[\"g2\"]}]}",
        );
        let config = load_config(&config, my_replacer);
        let expected = Config {
            builds: vec![
                BuildConfig::Bamboo {
                    server_url: String::from("http://my.bamboo"),
                    plan: String::from("MY-PLAN"),
                    token: Some(String::from("btoken")),
                },
                BuildConfig::CircleCI {
                    org: String::from("vankeisb"),
                    repo: String::from("react-tea-cup"),
                    branch: String::from("master"),
                    token: None,
                },
                BuildConfig::Travis {
                    server_url: String::from("https://my.travis"),
                    repository: String::from("my/repo"),
                    branch: String::from("develop"),
                    token: Some(String::from("")),
                },
            ],
        };
        assert_eq!(config.unwrap(), expected);
    }
}
