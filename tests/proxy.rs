extern crate environment;
extern crate hyper;
extern crate rand;
extern crate reqwest;
extern crate serde_json;

#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {

    use environment::Environment;

    use crate::common;
    use reqwest;
    use reqwest::StatusCode;
    use serde_json;
    use std::fs::{self, File};
    use std::io::{BufReader, Read};
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use trow::types::{HealthResponse, ReadinessResponse, RepoCatalog, RepoName, TagList};
    use trow_server::{digest, manifest};

    const TROW_ADDRESS: &str = "https://trow.test:8443";
    const DIST_API_HEADER: &str = "Docker-Distribution-API-Version";

    struct TrowInstance {
        pid: Child,
    }
    /// Call out to cargo to start trow.
    /// Seriously considering moving to docker run.

    async fn start_trow() -> TrowInstance {
        let mut child = Command::new("cargo")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
            .spawn()
            .expect("failed to start");

        let mut timeout = 100;

        let mut buf = Vec::new();
        File::open("./certs/domain.crt")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let cert = reqwest::Certificate::from_pem(&buf).unwrap();
        // get a client builder
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let mut response = client.get(TROW_ADDRESS).send().await;
        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::OK)) {
            thread::sleep(Duration::from_millis(100));
            response = client.get(TROW_ADDRESS).send().await;
            timeout -= 1;
        }
        if timeout == 0 {
            child.kill().unwrap();
            panic!("Failed to start Trow");
        }
        TrowInstance { pid: child }
    }

    impl Drop for TrowInstance {
        fn drop(&mut self) {
            common::kill_gracefully(&self.pid);
        }
    }

    async fn get_manifest(cl: &reqwest::Client, name: &str, tag: &str) {
        //Might need accept headers here
        let resp = cl
            .get(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let mani: manifest::ManifestV2 = resp.json().await.unwrap();
        assert_eq!(mani.schema_version, 2);
    }

    #[tokio::test]
    async fn test_runner() {
        //Need to start with empty repo
        fs::remove_dir_all("./data").unwrap_or(());

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let _trow = start_trow().await;

        let mut buf = Vec::new();
        File::open("./certs/domain.crt")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let cert = reqwest::Certificate::from_pem(&buf).unwrap();
        // get a client builder
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        //Using docker proxy should be able to download image even though it's not in registry
        get_manifest(&client, "docker/amouat/trow", "latest").await;
        get_manifest(&client, "docker/amouat/trow", "latest").await;

        get_manifest(&client, "docker/library/alpine", "latest").await;
        get_manifest(&client, "docker/library/alpine", "latest").await;

    }
}