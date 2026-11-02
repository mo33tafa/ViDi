# vidi

<p dir="rtl">
  <a href="README.md">🇬🇧 English</a>
  &nbsp;·&nbsp;
  <a href="README.fa.md">🇮🇷 فارسی</a>
</p>

`vidi` یک ابزار خط فرمان برای بررسی و عیب‌یابی سیستم‌های Linux و Windows است. این برنامه اطلاعات سیستم‌عامل، پردازش‌ها، سرویس‌ها، شبکه، سخت‌افزار و کاربران واردشده را جمع‌آوری می‌کند و نتیجه را به‌صورت گزارش خوانا یا JSON نمایش می‌دهد.

**توسعه‌دهنده:** Mostafa Heshmatpour

> **وضعیت پروژه:** vidi همچنان در حال توسعه است و قابلیت‌ها و بهبودهای بیشتری برای نسخه‌های آینده در نظر گرفته شده است.

## نصب

نسخه‌های آمادهٔ ۶۴ بیتی برای Linux و Windows در بخش Releases منتشر می‌شوند.

### Linux

در ترمینال اجرا کنید:

```sh
curl --fail --location https://raw.githubusercontent.com/mo33tafa/vidi/main/install.sh | sh
```

اگر دستور را با کاربر root اجرا کنید، برنامه در `/usr/local/bin` نصب می‌شود. در حالت عادی، محل نصب `~/.local/bin` است.

### Windows PowerShell

PowerShell را باز کنید و اجرا کنید:

```powershell
irm https://raw.githubusercontent.com/mo33tafa/vidi/main/install.ps1 | iex
```

برنامه در مسیر `%LOCALAPPDATA%\vidi\bin` نصب می‌شود و این مسیر به PATH کاربر اضافه خواهد شد. بعد از نصب، یک پنجرهٔ جدید PowerShell باز کنید.

## شروع سریع

```text
vidi --version
vidi get os
vidi get hardware
vidi get time
vidi get list
```

برای خروجی قابل استفاده در اسکریپت‌ها، گزینهٔ `--json` را اضافه کنید:

```text
vidi --json get hardware
vidi --json get time
```

## دستورات پرکاربرد

| دستور | کاربرد |
| --- | --- |
| `vidi get os` | سیستم‌عامل، کرنل، معماری، زمان بوت و مدت فعالیت |
| `vidi get time` | زمان محلی، UTC، منطقهٔ زمانی و وضعیت NTP |
| `vidi get hardware` | پردازنده، حافظه، Swap و دیسک‌ها |
| `vidi get hardware --full` | جزئیات بیشتر سخت‌افزار در صورت دسترس‌بودن |
| `vidi get process <pid>` | جزئیات یک پردازش |
| `vidi get service <name>` | جزئیات یک سرویس |
| `vidi get network` | رابط‌های شبکه و آمار ترافیک |
| `vidi get user` | کاربر فعلی |
| `vidi get list` | فهرست گزینه‌های ساده |

گروه‌های کامل‌تر نیز در دسترس هستند:

```text
system       اطلاعات سیستم‌عامل و متغیرهای محیطی
processes    فهرست، بررسی، جست‌وجو و توقف پردازش‌ها
services     فهرست و مدیریت سرویس‌ها
network      رابط‌ها، اتصال‌ها، مسیرها، DNS و پورت‌های در حال شنود
hardware     پردازنده، حافظه، ذخیره‌سازی، حسگرها و آداپتورها
users        حساب‌های محلی، نشست‌های فعال و گروه‌ها
```

نمونه‌های دیگر:

```text
vidi health
vidi diagnose all
vidi analyze processes
vidi processes list --count 20 --sort memory
vidi network listening
vidi snapshot --output snapshot.json
vidi hash /path/to/file
vidi investigate incident-2026
vidi monitor --duration 30 --interval 1000
```

## سطح دسترسی

برخی اطلاعات به اجرای ترمینال با دسترسی بالاتر نیاز دارند. برای بررسی پردازش‌های کاربران دیگر، مدیریت سرویس‌ها، خواندن جزئیات سخت‌افزار یا مشاهدهٔ اطلاعات محافظت‌شدهٔ شبکه از دسترسی root یا Administrator استفاده کنید.

در Linux:

```sh
sudo vidi services list
sudo vidi processes kill --pid 1234
```

در Windows، PowerShell را با گزینهٔ **Run as administrator** باز کنید.

اگر ابزار بومی موردنیاز روی سیستم نصب نباشد، vidi آن مقدار را در دسترس‌نبودن گزارش می‌کند و آن را به‌عنوان اطلاعات قطعی سیستم در نظر نمی‌گیرد.

## ساخت از سورس

برای build محلی به Rust نیاز دارید:

```sh
cargo build --release
cargo test
```

فایل اجرایی در یکی از مسیرهای زیر ساخته می‌شود:

```text
target/release/vidi
target/release/vidi.exe
```

## وضعیت توسعه

Linux پلتفرم اصلی و تست‌شدهٔ پروژه است. build مربوط به Windows در CI مخزن بررسی می‌شود، اما نتیجهٔ اطلاعات سخت‌افزار و سرویس‌ها به ابزارها و سطح دسترسی موجود روی هر سیستم بستگی دارد.

با push کردن tagهایی مانند `v0.1.0`، GitHub Actions به‌صورت خودکار آرشیوهای Linux و Windows را در بخش Releases ایجاد می‌کند.

## مجوز

MIT
