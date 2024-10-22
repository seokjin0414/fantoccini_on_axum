#![allow(non_snake_case)]

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
mod handlers {
    pub mod legacy_kepco {
        pub mod kepco;
        pub mod pp_kepco;
    }

    pub mod pp {
        pub mod user_info;
    }
}

mod models {
    pub mod response {
        pub mod commons;
    }

    pub mod error {
        pub mod response_errors;
        pub mod response_errors_def;
    }

    pub mod handler {
        pub mod legacy_kepco {
            pub mod kepco_models;
            pub mod pp_models;
        }

        pub mod pp {
            pub mod commons;
            pub mod user_info;
        }
    }

}

mod server_init {
    pub mod server_init;
}

use crate::server_init::server_init::server_initializer;

// 도쿄는 Axum 웹 프레임워크를 위한 비동기 런타임을 제공함. num_cpus 라이브러리를 사용하여 논리코어 개수에 따라 자동으로 thread pool 생성, request 분배함.
// Tokio is an asynchronous runtime, used here to run the Axum web framework. Automatically detects the number of logical cores to generate a thread pool of the appropraite size and distribute requests.
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let start: tokio::time::Instant = tokio::time::Instant::now();
    let server_start_time: DateTime<Utc> = Utc::now();

    // 로컬 테스팅을 위한 환경변수파일 로딩. 런타임 오류 발생할 수 있으니 EC2/ECS 배포시 반드시 비활성화!
    // An environment variable loader for local testing. Disable when distributing to AWS! Will result in a runtime error if not disabled.
    match dotenvy::dotenv() {
        Ok(path_buf) => {
            println!(
                "Env. variables at {} loaded: {:?}",
                path_buf.to_str().unwrap_or("N/A"),
                start.elapsed()
            );
        }
        Err(e) => {
            return Err(anyhow!(
                "Dotenvy could not load .env file: {}",
                e.to_string()
            ));
        }
    }

    // 유닛 테스트를 위하여 서버 시작 부분 논리는 분리해놓음
    // Server initialization logic separated for potential future unit testing.
    match server_initializer(start, server_start_time).await {
        Ok(server_initializer_result) => {
            println!(
                "Server successfully terminated: {}",
                server_initializer_result
            );
            return Ok(());
        }
        Err(e) => {
            return Err(anyhow!("Server exiting with error: {:?}", e));
        }
    }


    
}
