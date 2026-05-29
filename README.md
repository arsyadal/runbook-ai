# runbook-ai

Runbook AI adalah CLI untuk merekam proses debugging/development dengan AI coding agent, lalu menghasilkan dokumentasi operasional seperti runbook, changelog, dan postmortem.

## Fitur

- Merekam sesi kerja debugging/development.
- Menjalankan command melalui `rb exec` dan menyimpan output-nya.
- Menambahkan catatan keputusan, finding, todo, risk, workaround, dan root cause.
- Menangkap git diff sebelum dan sesudah sesi.
- Menghasilkan dokumen `RUNBOOK.md`, `CHANGELOG.md`, dan `POSTMORTEM.md`.

## Instalasi

Pastikan Rust sudah terpasang, lalu jalankan:

```bash
cargo build
```

Untuk menjalankan langsung:

```bash
cargo run -- --help
```

## Penggunaan

Inisialisasi storage Runbook di project:

```bash
cargo run -- init
```

Mulai sesi recording:

```bash
cargo run -- start "Fix bug login"
```

Jalankan command dan rekam hasilnya:

```bash
cargo run -- exec "cargo test"
```

Tambahkan catatan:

```bash
cargo run -- note --type decision "Gunakan pendekatan validasi input di layer CLI"
```

Lihat status sesi:

```bash
cargo run -- status
```

Selesaikan sesi:

```bash
cargo run -- stop
```

Generate dokumentasi:

```bash
cargo run -- generate all
```

## Struktur Output

Secara default, data sesi disimpan di `.rb/` dan dokumen hasil generate disimpan ke `docs/runbooks/`.

## Status

Project ini masih dalam tahap awal pengembangan.
