//! 화면 확인용 더미 데이터를 넣는다. `just seed`로 실행한다.

use std::path::PathBuf;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let Some(dir) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("사용법: seed <데이터 폴더>");
        return ExitCode::FAILURE;
    };
    match baeum_maru_lib::seed::run(&dir).await {
        Ok(report) => {
            println!("{}에 더미 데이터를 넣었습니다", dir.display());
            println!("관리자 PIN 123456, 다른 직원 PIN 1234");
            println!("관리자 복구 코드: {}", report.recovery_code);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
