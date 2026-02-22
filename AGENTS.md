# AGENTS.md
이 문서는 이 저장소에서 동작하는 에이전트 코딩 도구용 실행 기준입니다.
실제 코드/설정(`package.json`, `src-tauri/Cargo.toml`, `Claude.md`)을 기반으로 작성했습니다.

## 1) 우선 참고 문서
- `Claude.md` (핵심 개발 원칙과 커맨드)
- `README.md` (프로젝트 개요)
- `package.json` (프런트 스크립트)
- `src-tauri/Cargo.toml` (Rust 의존성/빌드 맥락)

추가 확인 결과:
- `.cursor/rules/` 없음
- `.cursorrules` 없음
- `.github/copilot-instructions.md` 없음
즉, Cursor/Copilot 전용 규칙은 현재 없으며 본 문서와 `Claude.md`를 따른다.

## 2) 프로젝트 맥락 요약
- 앱 성격: Tauri 기반 HTTP 클라이언트(포스트맨 유사)
- 프런트: SvelteKit 2 + Svelte 5 runes + Tailwind 4 + Skeleton
- 백엔드: Rust + Tauri 2
- 타입 브리지: `tauri-specta`로 `src/lib/bindings.ts` 생성
- 패키지 매니저: Bun

## 3) Build/Lint/Test 명령어
### 루트 명령
```bash
bun install
bun run dev
bun run check
bun run build
bun run tauri dev
bun run tauri build
```

### Rust 명령 (`src-tauri`에서 실행)
```bash
cargo check
cargo fmt
cargo fmt --check
cargo clippy
cargo test
```

### 권장 검증 시퀀스
```bash
cd src-tauri && cargo fmt --check && cargo clippy && cargo test
bun run check
```

## 4) 단일 테스트 실행 (중요)
현재 저장소에 테스트 파일이 많지 않거나 없을 수 있으므로, 아래는 표준 패턴이다.

### Rust 단일 테스트
```bash
# 테스트 이름 일부 매칭
cd src-tauri && cargo test test_name_substring

# 특정 integration test 파일에서 하나 실행
cd src-tauri && cargo test --test http_client test_name_substring

# 로그 확인 필요 시
cd src-tauri && cargo test test_name_substring -- --nocapture
```

### 프런트엔드 테스트
- `package.json`에 `test`/`vitest`/`playwright` 스크립트가 없다.
- 따라서 프런트 단일 테스트 명령은 현재 기준으로 제공되지 않는다.
- 프런트 품질 게이트는 `bun run check`를 사용한다.

## 5) 에이전트 작업 절차
1. 관련 코드 먼저 읽고 기존 패턴 파악
2. 최소 범위로 구현
3. Rust 변경 시 `fmt/clippy/test` 실행
4. 프런트 변경 시 `bun run check` 실행
5. Tauri command 시그니처 변경 시 바인딩 갱신 확인 (`bun run tauri dev`)
6. 불필요한 리팩터링/대규모 포맷 변경 금지

## 6) 코드 스타일: 공통
- KISS 유지: 작은 함수, 명확한 이름, 얕은 추상화
- 기능 변경과 스타일 변경을 한 번에 섞지 않기
- 주석은 비직관 로직에만 최소로 작성
- 매직값 반복 시 상수화
- 기존 파일 스타일을 우선적으로 존중

## 7) 코드 스타일: Svelte/TypeScript
### 포맷
- 문자열: 큰따옴표
- 세미콜론: Svelte/TS 코드에서는 사용하지 않음
- 들여쓰기: 2칸
- 상태 관리: Svelte 5 runes(`$state`, `$derived`) 패턴 유지

### import 관례
- 권장 순서:
  1) `svelte` 같은 프레임워크 코어
  2) `$lib/...` 내부 모듈
  3) `@tauri-apps/...` 플러그인
  4) 기타 서드파티
- 동일 파일에서 import 스타일 일관성 유지

### 타입/상태
- 인터페이스/유니온 타입을 명시적으로 선언
- IPC/스토리지 경계 데이터는 정규화 후 사용
- 키 이름(`templates`, `history`, `theme`)은 일관되게 유지
- 접근성 관련 속성(`role`, `aria-*`, 키보드 이벤트) 제거 금지

## 8) 코드 스타일: Rust/Tauri
### 포맷/정적 분석
- `cargo fmt` 결과를 기준으로 정렬/개행 유지
- `cargo clippy` 경고를 가능한 해결, `allow` 남용 금지

### 네이밍
- 파일/모듈/함수: `snake_case`
- 타입/enum/trait: `PascalCase`
- 상수: `UPPER_SNAKE_CASE`

### 타입/직렬화
- 프런트 노출 타입은 `serde` + `specta::Type` 파생 유지
- JSON 필드명은 `#[serde(rename_all = "camelCase")]` 유지

### Tauri command
- `#[tauri::command]`와 `#[specta::specta]`를 함께 사용
- 신규 command는 `src-tauri/src/lib.rs`의 `collect_commands![]`에 등록
- command 변경 후 `src/lib/bindings.ts` 동기화 확인

## 9) 에러 처리/로깅
- Rust 커스텀 에러는 `thiserror` enum 사용
- `Result<T, E>` 중심으로 전파하고 문자열 에러 남발 금지
- 외부 입력/HTTP/파일 I/O는 명시적으로 검증
- 로깅은 `src-tauri/src/modules/logger.rs` 패턴 우선
- 에러 메시지에 문맥(context) 포함

## 10) 자동 생성 파일 규칙
- `src/lib/bindings.ts`는 자동 생성 파일이므로 수동 편집 금지
- 타입 불일치가 보이면 바인딩 재생성 여부 먼저 점검

## 11) 커밋/PR 전 체크리스트
- [ ] 변경 범위가 요청사항과 정확히 일치하는가
- [ ] Rust: `cargo fmt --check`, `cargo clippy`, `cargo test` 통과
- [ ] Frontend: `bun run check` 통과
- [ ] Tauri command 변경 시 바인딩 동기화 확인
- [ ] 민감정보(토큰/키/로컬 비밀값) 포함 여부 확인

## 12) 금지/주의 사항
- 사용자 변경사항을 임의로 되돌리지 않는다.
- 요청 없는 파괴적 Git 명령(`reset --hard`, 강제 푸시) 금지.
- 요청 없는 의존성 추가/대규모 구조 변경 금지.
- 문서가 코드와 어긋나면 문서를 즉시 갱신한다.

## 13) 문서 동기화 규칙
- 새 패턴이 생기면 `Claude.md`와 `AGENTS.md`를 함께 업데이트한다.
- 추후 Cursor/Copilot 규칙 파일이 추가되면 본 문서에 반영한다.

이 문서는 실행용 가이드다.
코드의 실제 상태를 우선하고, 변경 시 이 문서를 함께 유지보수한다.
