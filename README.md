# CubeXCraft Installer

Устанавливает CubeXCraft на iPhone и iPad с компьютера на macOS или Windows. Всё делает одна кнопка:

- скачивает свежую сборку CubeXCraft с сайта и подписывает её вашим Apple ID с правом Increased Memory Limit;
- ставит StikDebug для JIT и кладёт в него файл сопряжения;
- живёт в трее и раз в несколько дней сам продлевает подпись — бесплатный Apple ID подписывает на 7 дней.

Скачать: https://www.cubexcraft.site — окно «Скачать» → iPhone.

## Сборка

Нужны [Bun](https://bun.sh) и Rust.

```bash
bun i
bun tauri build
```

Готовые сборки для macOS (universal, .dmg) и Windows (NSIS) собирает GitHub Actions: `.github/workflows/build.yml`.

## Лицензия

С версии 1.4.7 код установщика открыт **только для просмотра**: смотреть, проверять и собирать для себя можно, копировать, распространять и делать на его основе свои установщики — нельзя, см. [LICENSE](LICENSE). Основа — [iloader](https://github.com/nab138/iloader) (© 2025 nab138), её части остаются под MIT — [LICENSE-MIT](LICENSE-MIT). Версии до 1.4.6 включительно вышли под MIT. Сторонние компоненты — [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), уязвимости — [SECURITY.md](SECURITY.md).

Since 1.4.7 the installer source is published for review only (see LICENSE); the parts from iloader by nab138 remain under MIT (LICENSE-MIT).
