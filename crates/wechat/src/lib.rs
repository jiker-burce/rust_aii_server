use redis::Commands;
use redis::FromRedisValue;
use reqwest::blocking::multipart;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub mod mini_program {
    use super::*;
    pub struct MiniProgram;

    #[derive(Debug)]
    struct CustomError(String);

    impl fmt::Display for CustomError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl Error for CustomError {}

    impl MiniProgram {
        fn get_redis_conn() -> redis::RedisResult<redis::Connection> {
            let client = redis::Client::open("redis://127.0.0.1/")?;
            let connection = client.get_connection();
            connection
        }

        // https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/mp-access-token/getAccessToken.html
        // 获取文件信息时报错:  {"errcode":40001,"errmsg":"invalid credential, access_token is invalid or not latest, could get access_token by getStableAccessToken, more details at https://mmbizurl.cn/s/JtxxFh33r rid: 6465d38b-7c92cbd3-6b53fb15"}
        // 更换地址为: https://api.weixin.qq.com/cgi-bin/stable_token
        pub async fn get_access_token() -> Result<String, Box<dyn Error>> {
            let app_id = dotenvy::var("MINI_APP_ID")?;
            let secret = dotenvy::var("MINI_APP_SECRET")?;
            let mut redis_conn = MiniProgram::get_redis_conn()?;
            let r_access_token = redis::cmd("GET")
                .arg("aii_server:mini:access_token")
                .query::<redis::Value>(&mut redis_conn)?;

            if redis::Value::Nil != r_access_token {
                return Ok(String::from_redis_value(&r_access_token)?);
            }

            let url = format!(
                "https://api.weixin.qq.com/cgi-bin/token?appid={}&secret={}&grant_type={}",
                app_id, secret, "client_credential"
            );
            let res = Client::new().get(url).send().await?;
            println!("new token");
            let res = res.text().await?;
            let value = serde_json::from_str::<Value>(&res);

            let  Ok(v) = value else {
                return Err(Box::new(CustomError("微信没有正常返回数据".to_string())));
            };

            let access_token = v.get("access_token");
            let  Some(token) = access_token else {
                let err_msg = format!("微信返回值异常: {}", v);
                return Err(Box::new(CustomError(err_msg)));
            };

            let result_token = token.to_string().replace("\"", "");

            redis::cmd("SET")
                .arg("aii_server:mini:access_token")
                .arg(result_token.clone())
                .query(&mut redis_conn)?;
            _ = redis_conn.expire::<String, i32>("aii_server:mini:access_token".to_string(), 7000);

            Ok(result_token)
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct UploadFile {
        pub file_id: String,
        pub url: String,
        pub errcode: i32,
        pub errmsg: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FileListRequest {
        pub env: String,
        #[serde(rename = "file_list")]
        pub file_list: Vec<FileList>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FileList {
        pub fileid: String,
        #[serde(rename = "max_age")]
        pub max_age: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DeleteFileListRequest {
        pub env: String,
        #[serde(rename = "fileid_list")]
        pub fileid_list: Vec<String>,
    }

    // 对象存储操作接口文档: https://developers.weixin.qq.com/miniprogram/dev/wxcloudrun/src/development/storage/service/
    // poem 文件上传: https://rustmagazine.github.io/rust_magazine_2021/chapter_9/poem-openapi.html
    impl UploadFile {
        // 上传文件前需要先通过一个接口获取文件上传连接信息
        pub async fn get_uplaod_info(cloud_file_key: String) -> Result<Value, Box<dyn Error>> {
            let access_token = MiniProgram::get_access_token()
                .await
                .expect("获取token异常");

            let url = format!(
                "https://api.weixin.qq.com/tcb/uploadfile?access_token={}",
                access_token
            );

            let mut map = HashMap::new();
            map.insert("env", "dev-3g8j7o151d57d023".to_string());
            map.insert("path", cloud_file_key);

            let res = Client::new()
                .post(url)
                .header("content-type", "application/json; charset=utf-8")
                .json(&map)
                .send()
                .await?;

            let res = res.text().await?;
            let value = serde_json::from_str::<Value>(&res).expect("微信没有正常返回数据");

            Ok(value)
        }

        // 依据上面的链接信息传送文件具体内容
        pub fn upload_file(
            value: Value,
            cloud_file_key: String,
            absolute_file_path: String,
        ) -> Result<Self, Box<dyn Error>> {
            let form = multipart::Form::new()
                .text("key", cloud_file_key)
                .text(
                    "Signature",
                    value
                        .get("authorization")
                        .unwrap()
                        .to_string()
                        .replace("\"", ""),
                )
                .text(
                    "x-cos-security-token",
                    value.get("token").unwrap().to_string().replace("\"", ""),
                )
                .text(
                    "x-cos-meta-fileid",
                    value
                        .get("cos_file_id")
                        .unwrap()
                        .to_string()
                        .replace("\"", ""),
                )
                // And a file...
                .file("file", absolute_file_path)?;

            // And finally, send the form
            let client = reqwest::blocking::Client::new();

            // TODO: 研究如何从 send 返回值进行 serde_json 反序列化
            let _ = client
                .post(value.get("url").unwrap().to_string().replace("\"", ""))
                .multipart(form)
                .send()
                .expect("上传失败");

            let upload_file = UploadFile {
                file_id: value.get("file_id").unwrap().to_string().replace("\"", ""),
                url: value.get("url").unwrap().to_string().replace("\"", ""),
                errcode: 0,
                errmsg: "ok".to_string(),
            };

            Ok(upload_file)
        }

        pub async fn get_files(upload_files: Vec<UploadFile>) -> Result<Value, Box<dyn Error>> {
            let file_list = upload_files.iter().fold(Vec::new(), |mut files, file| {
                files.push(FileList {
                    fileid: file.file_id.clone(),
                    max_age: 7200,
                });

                files
            });

            let file_list_request = FileListRequest {
                env: "dev-3g8j7o151d57d023".to_string(),
                file_list: file_list,
            };

            let access_token = MiniProgram::get_access_token()
                .await
                .expect("获取token异常");

            let url = format!(
                "https://api.weixin.qq.com/tcb/batchdownloadfile?access_token={}",
                access_token
            );

            let res = Client::new()
                .post(url)
                .json(&file_list_request)
                .send()
                .await?;

            let res = res.text().await?;
            // println!("批量文件返回结果 res: {:#?}", res);
            let value = serde_json::from_str::<Value>(&res)?;

            Ok(value)
        }

        pub async fn delete_files(upload_files: Vec<UploadFile>) -> Result<Value, Box<dyn Error>> {
            let file_list = upload_files.iter().fold(Vec::new(), |mut files, file| {
                files.push(file.file_id.clone());

                files
            });

            let file_list_request = DeleteFileListRequest {
                env: "dev-3g8j7o151d57d023".to_string(),
                fileid_list: file_list,
            };

            let access_token = MiniProgram::get_access_token()
                .await
                .expect("获取token异常");

            let url = format!(
                "https://api.weixin.qq.com/tcb/batchdeletefile?access_token={}",
                access_token
            );

            let res = Client::new()
                .post(url)
                .json(&file_list_request)
                .send()
                .await?;

            let res = res.text().await?;
            // println!("批量文件返回结果 res: {:#?}", res);
            let value = serde_json::from_str::<Value>(&res)?;

            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{println, vec};

    use crate::mini_program::UploadFile;

    use super::*;

    #[test]
    fn test_get_access_token() {
        tokio_test::block_on(async {
            println!("dotenvy: {}", dotenvy::dotenv().unwrap().display());
            let token = mini_program::MiniProgram::get_access_token()
                .await
                .expect("微信access_token获取失败");
            println!("token: {}", token);
        });
    }

    #[test]
    fn test_get_uplaod_info() {
        let cloud_file_key = "assets/images/69cb3f638018703404662f011c528dad1.jpg".to_string();

        tokio_test::block_on(async {
            let upload_file = mini_program::UploadFile::get_uplaod_info(cloud_file_key)
                .await
                .expect("微信获取文件上传信息失败");
            println!("upload_file: {:?}", upload_file);
        });
    }

    #[test]
    fn test_upload_file() {
        let cloud_file_key = "assets/images/69cb3f638018703404662f011c528dad1.jpg".to_string();
        let absolute_file_path = "/Users/gujinhe/my-tool/rust/canstory/aii_server/assets/images/69cb3f638018703404662f011c528dad1.jpg".to_string();

        let upload_file_value = tokio_test::block_on(async {
            let upload_file_value =
                mini_program::UploadFile::get_uplaod_info(cloud_file_key.clone())
                    .await
                    .expect("获取文件上传信息失败");
            upload_file_value
        });

        // 不能将阻塞的请求放入 上面的block_on()方法体中,否则会报下面异常
        // Cannot drop a runtime in a context where blocking is not allowed. This happens when a runtime is dropped from within an asynchronous context
        let result = mini_program::UploadFile::upload_file(
            upload_file_value,
            cloud_file_key,
            absolute_file_path,
        )
        .expect("上传文件失败");

        println!("file: {:?}", result);
    }

    #[test]
    fn test_get_files() {
        let upload_files = vec![
            UploadFile{
                file_id: "cloud://dev-3g8j7o151d57d023.6465-dev-3g8j7o151d57d023-1318257894/assets/images/69cb3f638018703404662f011c528dad1.jpg".to_string(),
                url: "https://cos.ap-shanghai.myqcloud.com/6465-dev-3g8j7o151d57d023-1318257894/assets/images/69cb3f638018703404662f011c528dad1.jpg".to_string(),
                errcode:0,
                errmsg: "ok".to_string()
            }
        ];

        tokio_test::block_on(async {
            let batch_files = mini_program::UploadFile::get_files(upload_files)
                .await
                .expect("批量获取文件失败");
            println!("批量获取文件结果: {:?}", batch_files);
        });
    }

    #[test]
    fn test_batch_delete_files() {
        let upload_files = vec![
            UploadFile{
                file_id: "cloud://dev-3g8j7o151d57d023.6465-dev-3g8j7o151d57d023-1318257894/assets/images/69cb3f638018703404662f011c528dad1.jpg".to_string(),
                url: "https://cos.ap-shanghai.myqcloud.com/6465-dev-3g8j7o151d57d023-1318257894/assets/images/69cb3f638018703404662f011c528dad1.jpg".to_string(),
                errcode:0,
                errmsg: "ok".to_string()
            }
        ];

        tokio_test::block_on(async {
            let batch_files = mini_program::UploadFile::delete_files(upload_files)
                .await
                .expect("批量删除文件失败");
            println!("批量删除文件结果: {:?}", batch_files);
        });
    }
}
