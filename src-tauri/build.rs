fn main() {
    // 구조 변경 파일이 바뀌면 다시 빌드해서 실행 파일에 넣는다
    println!("cargo:rerun-if-changed=migrations");
    tauri_build::build();
}
