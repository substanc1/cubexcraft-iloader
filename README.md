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

Код — MIT, см. [LICENSE](LICENSE). Шрифты Unbounded, JetBrains Mono и Manrope — SIL OFL 1.1.
