# Axum + Fantoccini 기반 KEPCO 크롤링 API

이 프로젝트는 **Rust의 [Axum](https://github.com/tokio-rs/axum) 웹 프레임워크**와  
**[Fantoccini](https://github.com/jonhoo/fantoccini) WebDriver 클라이언트**를 사용하여  
**한전 파워플래너, 한전온(KEPCO PowerPlanner & KEPCO ON)** 사이트를 크롤링하는 API 서버 예제입니다.

---

## 특징

- **최신 Chrome & ChromeDriver 지원**  
  크롤링 환경은 **최신 버전의 크롬/크롬드라이버**에서 동작합니다.
- **서버 부팅 시점 ChromeDriver 자동 런칭**  
  서버 시작 시점에 ChromeDriver를 자동 실행합니다.  
  크롤링 요청마다 드라이버를 매번 띄우지 않아 **응답 지연을 최소화**합니다.
- **병렬 크롤링 지원**  
  `Arc<Client>` 패턴으로 **동시 크롤링 요청**이 가능합니다.
- **Legacy 코드 분리**  
  구버전(legacy) 처리 코드는 별도로 관리합니다.

---

## 주요 코드 및 환경

### 서버 시작 시 ChromeDriver 런칭

```rust
let _chromedriver = start_chromedriver()
    .await
    .map_err(|e| anyhow!("Failed to start chromedriver: {}", e))?;
```
- 서버 시작과 동시에 ChromeDriver 프로세스를 실행해, 크롤링 요청 시 즉시 활용합니다.

### 병렬 크롤링을 위한 Arc<Client> 및 환경/의존성 관리
```rust
pub async fn create_client(url: &str, test: bool) -> Result<Arc<Client>> {
    let client = ClientBuilder::native()
        .capabilities(create_capabilities(test)?)
        .connect(url)
        .await
        .map_err(|e| {
            eprintln!("Failed to connect process: {:?}", e);
            anyhow!("Failed to connect process: {:?}", e)
        })?;

    Ok(Arc::new(client))
}
```
- WebDriver 클라이언트를 Arc로 관리하여 여러 비동기 작업에서 안전하게 병렬 크롤링이 가능합니다.

### 환경
- Chrome, ChromeDriver 최신 버전을 사전에 설치해주세요.
- 환경 변수(.env)는 필요에 따라 자유롭게 수정할 수 있습니다.
- 프로젝트에서 사용하는 라이브러리/의존성 정보는 Cargo.toml 파일을 참고하세요.

---

## 문의

질문이 있거나, 추가 지원이 필요하시거나, 협업을 원하신다면 아래로 연락해 주세요.

- **이메일:** sars21@hanmail.net  
- **LinkedIn:** [https://www.linkedin.com/in/seokjin-shin/](https://www.linkedin.com/in/seokjin-shin/)

언제든 편하게 문의 바랍니다!
