# ⚡ Tiket War Bot

Ultra-fast ticket war bot untuk konser di tiket.com — 100% Rust.

```
 ████████╗██╗██╗  ██╗███████╗████████╗    ██╗    ██╗ █████╗ ██████╗
 ╚══██╔══╝██║██║ ██╔╝██╔════╝╚══██╔══╝    ██║    ██║██╔══██╗██╔══██╗
    ██║   ██║█████╔╝ █████╗     ██║       ██║ █╗ ██║███████║██████╔╝
    ██║   ██║██╔═██╗ ██╔══╝     ██║       ██║███╗██║██╔══██║██╔══██╗
    ██║   ██║██║  ██╗███████╗   ██║       ╚███╔███╔╝██║  ██║██║  ██║
    ╚═╝   ╚═╝╚═╝  ╚═╝╚══════╝   ╚═╝        ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝
```

## 🚀 Features

- **Ultra-fast** — Rust + Chrome DevTools Protocol, near-zero overhead
- **Auto-fill** — Pre-built JavaScript injection, semua field diisi dalam satu atomic operation
- **Smart monitor** — High-frequency polling (100ms) deteksi tiket available
- **Auto-checkout** — 8-step automated checkout flow
- **CAPTCHA aware** — Pause otomatis kalau ada CAPTCHA, alert ke user + Telegram
- **Telegram notifications** — Real-time alert via Telegram bot
- **Speed benchmark** — Test latency sebelum war
- **Configurable selectors** — CSS selector bisa disesuaikan kalau layout berubah

## 📋 Requirements

- **Rust** 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Chrome / Chromium** terinstall di sistem
- **Telegram Bot Token** (optional, untuk notifications)

## 🔧 Installation

```bash
# Clone repo
git clone <repo-url>
cd tiket-war-bot

# Build release (optimized)
cargo build --release

# Binary ada di:
./target/release/tiket-war-bot
```

## ⚙️ Configuration

```bash
# Copy example config
cp config.example.toml config.toml

# Edit sesuai kebutuhan
nano config.toml
```

### Config sections:

| Section | Keterangan |
|---------|-----------|
| `[target]` | URL event, kategori tiket, jumlah, max harga |
| `[buyer]` | Nama, email, phone, nomor KTP |
| `[payment]` | Method: `virtual_account`, `credit_card`, `gopay`, atau `bliblipay` |
| `[monitor]` | Poll interval, waktu mulai war, pre-warm |
| `[browser]` | Chrome path, headless mode, user data dir |
| `[telegram]` | Bot token & chat ID |
| `[speed]` | Parallel tabs, pre-connect |
| `[selectors]` | CSS selectors untuk semua elemen halaman |

## 🎮 Usage

### 1. Validate Config
```bash
./target/release/tiket-war-bot config
```

### 2. Warm Up (Login)
```bash
./target/release/tiket-war-bot warmup
```
Browser terbuka → login ke tiket.com → `Ctrl+C` setelah selesai. Session tersimpan.

### 3. Speed Benchmark
```bash
./target/release/tiket-war-bot bench
```
Test latency koneksi, JS eval, page load, dan click speed.

### 4. WAR MODE 🔥
```bash
./target/release/tiket-war-bot war
```

**War flow:**
1. Load config → launch Chrome
2. Pre-warm connections
3. Navigate ke halaman target
4. Countdown sampai `start_time`
5. Auto-refresh saat war time
6. Monitor tiket (polling setiap 100ms)
7. Tiket terdeteksi → auto-checkout 8 step
8. Telegram notification

## ⚡ Speed Optimizations

| Teknik | Detail |
|--------|--------|
| Pre-built JS | Autofill JavaScript di-compile saat startup |
| Atomic fill | Semua form field diisi dalam 1x JS eval |
| Chrome flags | 20+ flags matikan fitur yang tidak perlu |
| Native setter | Bypass React/Vue virtual DOM |
| CDP direct | Chrome DevTools Protocol langsung, bukan WebDriver |
| Release profile | LTO, single codegen unit, stripped binary |

## 📁 Project Structure

```
├── Cargo.toml                 # Dependencies & build config
├── config.example.toml        # Example configuration
├── README.md
├── .gitignore
└── src/
    ├── main.rs                # CLI entry point
    ├── config.rs              # Config loading & validation
    ├── error.rs               # Error types
    ├── browser.rs             # Chrome CDP management
    ├── monitor.rs             # Ticket availability monitor
    ├── autofill.rs            # Form auto-fill engine
    ├── checkout.rs            # Checkout flow orchestrator
    ├── notification.rs        # Telegram notifications
    └── speed.rs               # Speed benchmarking
```

## ⚠️ Tips

- **Test dulu** dengan event yang sudah available sebelum war beneran
- **Sesuaikan CSS selectors** di `[selectors]` config kalau layout tiket.com berubah
- **Pakai VPS di Jakarta** untuk latency minimal ke server tiket.com
- **Run `warmup` dulu** supaya session login tersimpan
- **Run `bench`** untuk cek kecepatan koneksi kamu

## 📄 License

MIT
