// CubeXCraft (windows-claude, 25.10): «Как установить?» — по шагам, на каждом схема с номерами (что нажимать по
// порядку) и стрелками. Владелец: «инструкцию с картинками и стрелочками, что и куда и в какой последовательности».
// Схемы, а не скриншоты: снимков iPhone у нас нет, а схема не устаревает с новой iOS. Тексты — как в «Как установить.txt».
import { useState, type ReactNode } from "react";
import { Modal } from "./components/Modal";

type Lang = "ru" | "en";
type Platform = "mac" | "windows" | "linux";

const C = {
  text: "#f0e2e3",
  mute: "#8a6167",
  line: "#4a1018",
  red: "#ff2a3c",
  yel: "#fcee0a",
  ok: "#00ff9f",
  ios: "#000000",
  iosRow: "#1c1c1e",
  iosSep: "#38383a",
  iosMute: "#8e8e93",
  iosBlue: "#0a84ff",
  alert: "#2c2c2e",
};
const MONO = "'JetBrains Mono', monospace";
const SANS = "'Manrope', -apple-system, 'Segoe UI', sans-serif";

const T = ({ x, y, s = 9, c = "#ffffff", a = "start", w = 400, f = SANS, children }: any) => (
  <text x={x} y={y} fontSize={s} fill={c} textAnchor={a} fontWeight={w} fontFamily={f}>
    {children}
  </text>
);

// жёлтый кружок с номером — что нажимать по порядку
const Num = ({ x, y, n }: any) => (
  <g>
    <circle cx={x} cy={y} r={9} fill={C.yel} />
    <text x={x} y={y + 3.5} fontSize={10} fontWeight={800} textAnchor="middle" fill="#120308" fontFamily={MONO}>
      {n}
    </text>
  </g>
);

const Arrow = ({ d }: any) => (
  <path d={d} fill="none" stroke={C.yel} strokeWidth={2.2} strokeLinecap="round" markerEnd="url(#cxh-ah)" />
);

// рамка iPhone с заголовком экрана
const Phone = ({ x, y = 8, w = 150, h = 234, title, title2, bg = C.ios }: any) => (
  <g>
    <rect x={x} y={y} width={w} height={h} rx={18} fill={bg} stroke="#3a3a3c" strokeWidth={2} />
    <rect x={x + w / 2 - 22} y={y + 7} width={44} height={10} rx={5} fill="#1c1c1e" />
    {title && (
      <T x={x + 12} y={y + 40} s={title2 ? 10 : 12} w={700}>
        {title}
      </T>
    )}
    {title2 && (
      <T x={x + 12} y={y + 52} s={10} w={700}>
        {title2}
      </T>
    )}
  </g>
);

// строка настроек iOS; hi — обведена: сюда нажать
const Row = ({ x, y, w = 134, label, hi, value, toggle }: any) => (
  <g>
    <rect x={x} y={y} width={w} height={21} rx={hi ? 5 : 0} fill={C.iosRow} stroke={hi ? C.yel : "none"} strokeWidth={hi ? 2 : 0} />
    <T x={x + 7} y={y + 14} s={7.8}>
      {label}
    </T>
    {toggle !== undefined ? (
      <g>
        <rect x={x + w - 26} y={y + 4} width={20} height={13} rx={6.5} fill={toggle ? "#30d158" : "#3a3a3c"} />
        <circle cx={toggle ? x + w - 12.5 : x + w - 19.5} cy={y + 10.5} r={5} fill="#ffffff" />
      </g>
    ) : (
      <T x={x + w - 6} y={y + 14} s={8} c={C.iosMute} a="end">
        {value ? value + " ›" : "›"}
      </T>
    )}
  </g>
);

// окно-вопрос iOS; кнопки в столбик, hi — какую нажать
const Alert = ({ x, y, w = 134, lines, sub = [], buttons, hi = 0 }: any) => {
  const body = 14 + lines.length * 11 + sub.length * 10 + 6;
  return (
    <g>
      <rect x={x} y={y} width={w} height={body + buttons.length * 21} rx={10} fill={C.alert} />
      {lines.map((l: string, i: number) => (
        <T key={"l" + i} x={x + w / 2} y={y + 16 + i * 11} s={8.5} w={700} a="middle">
          {l}
        </T>
      ))}
      {sub.map((l: string, i: number) => (
        <T key={"s" + i} x={x + w / 2} y={y + 16 + lines.length * 11 + i * 10} s={7} c={C.iosMute} a="middle">
          {l}
        </T>
      ))}
      {buttons.map((b: string, i: number) => {
        const by = y + body + i * 21;
        return (
          <g key={"b" + i}>
            <line x1={x} y1={by} x2={x + w} y2={by} stroke={C.iosSep} />
            {i === hi && <rect x={x + 3} y={by + 2} width={w - 6} height={17} rx={5} fill="none" stroke={C.yel} strokeWidth={2} />}
            <T x={x + w / 2} y={by + 14} s={8.5} c={C.iosBlue} w={i === hi ? 700 : 400} a="middle">
              {b}
            </T>
          </g>
        );
      })}
    </g>
  );
};

// окно на компьютере
const Win = ({ x, y, w, h, title, mac, fill = "#0f0709" }: any) => (
  <g>
    <rect x={x} y={y} width={w} height={h} rx={8} fill={fill} stroke={C.line} />
    <path d={`M${x} ${y + 20}V${y + 8}a8 8 0 0 1 8 -8H${x + w - 8}a8 8 0 0 1 8 8V${y + 20}Z`} fill="#00000055" />
    {mac && [0, 1, 2].map((i) => <circle key={i} cx={x + 11 + i * 11} cy={y + 10} r={3.5} fill={["#ff5f57", "#febc2e", "#28c840"][i]} />)}
    {title && (
      <T x={x + (mac ? 46 : 9)} y={y + 13.5} s={7.5} c="#c4a8ac" f={MONO}>
        {title}
      </T>
    )}
  </g>
);

const Btn = ({ x, y, w, h = 20, label, hi, red, blue }: any) => (
  <g>
    <rect
      x={x}
      y={y}
      width={w}
      height={h}
      rx={red ? 0 : 4}
      fill={red ? "#ff2a3c26" : blue ? "#0a84ff" : "#2a1015"}
      stroke={hi ? C.yel : red ? C.red : C.line}
      strokeWidth={hi ? 2 : 1}
    />
    <T x={x + w / 2} y={y + h / 2 + 3.2} s={red ? 8 : 8.5} a="middle" w={700} c={red ? "#ffd6d9" : "#ffffff"} f={red ? MONO : SANS}>
      {label}
    </T>
  </g>
);

const Art = ({ children }: { children: ReactNode }) => (
  <svg viewBox="0 0 520 250" className="cxh-art" role="img">
    <defs>
      <marker id="cxh-ah" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0 0 10 5 0 10z" fill={C.yel} />
      </marker>
    </defs>
    {children}
  </svg>
);

// установщик схемой: шапка, метка блока
const Installer = ({ x, y, w, h }: any) => (
  <g>
    <Win x={x} y={y} w={w} h={h} title="CubeXCraft Installer" />
    <T x={x + 14} y={y + 40} s={10} w={800} c="#f7eaea" f={MONO}>
      CUBE<tspan fill={C.red}>X</tspan>CRAFT
    </T>
  </g>
);

const Kick = ({ x, y, w, label, yellow }: any) => (
  <g>
    <T x={x} y={y} s={7.5} c={yellow ? C.yel : C.red} f={MONO}>
      {label}
    </T>
    <line x1={x + label.length * 4.9 + 6} y1={y - 3} x2={x + w} y2={y - 3} stroke={yellow ? "#fcee0a55" : "#ff2a3c55"} />
  </g>
);

const QR = ({ x, y, size }: any) => {
  const cells: ReactNode[] = [];
  const n = 9;
  const c = size / n;
  for (let i = 0; i < n; i++)
    for (let j = 0; j < n; j++) {
      const corner = (i < 3 && j < 3) || (i < 3 && j > 5) || (i > 5 && j < 3);
      const on = corner ? !(i % 6 === 1 && j % 6 === 1) : (i * 7 + j * 3 + i * j) % 3 === 0;
      if (on) cells.push(<rect key={i + "-" + j} x={x + j * c} y={y + i * c} width={c} height={c} fill="#ffffff" />);
    }
  return (
    <g>
      <rect x={x - 4} y={y - 4} width={size + 8} height={size + 8} fill="#120308" />
      {cells}
    </g>
  );
};

type Step = { title: string; text: string; art: ReactNode };

function steps(lang: Lang, platform: Platform): Step[] {
  const ru = lang === "ru";
  const L = (r: string, e: string) => (ru ? r : e);
  const first: Step =
    platform === "mac"
      ? {
          title: L("Первый запуск на Mac", "First launch on Mac"),
          text: L(
            "Mac сразу не откроет установщик: «Не удалось проверить» — нажмите «Готово». Затем Системные настройки → «Конфиденциальность и безопасность» → внизу «Всё равно открыть» → пароль Mac. Это нужно один раз.",
            "The Mac won't open the installer right away: “couldn't be verified” — click “Done”. Then System Settings → Privacy & Security → at the bottom “Open Anyway” → your Mac password. Needed only once.",
          ),
          art: (
            <Art>
              <Alert
                x={14}
                y={46}
                w={196}
                lines={[L("«CubeXCraft Installer»", "“CubeXCraft Installer”"), L("не открыт", "Not Opened")]}
                sub={[L("Apple не удалось проверить,", "Apple could not verify it"), L("что в нём нет вредоносного ПО", "is free of malware")]}
                buttons={[L("Готово", "Done"), L("Переместить в Корзину", "Move to Trash")]}
              />
              <Num x={14} y={124} n={1} />
              <Arrow d="M214 124 C 240 124, 240 120, 262 120" />
              <Win x={268} y={10} w={240} h={230} title={L("Конфиденциальность и безопасность", "Privacy & Security")} mac />
              <T x={282} y={48} s={10} w={700}>
                {L("Безопасность", "Security")}
              </T>
              <rect x={278} y={58} width={220} height={96} rx={6} fill="#1a0b0f" />
              <T x={288} y={78} s={7.8} c="#c4a8ac">
                {L("«CubeXCraft Installer» заблокирован,", "“CubeXCraft Installer” was blocked")}
              </T>
              <T x={288} y={90} s={7.8} c="#c4a8ac">
                {L("так как это ПО от неустановленного", "to protect your Mac.")}
              </T>
              <T x={288} y={102} s={7.8} c="#c4a8ac">
                {L("разработчика.", "")}
              </T>
              <Btn x={376} y={122} w={112} label={L("Всё равно открыть", "Open Anyway")} hi />
              <Num x={370} y={132} n={2} />
              <rect x={318} y={176} width={140} height={48} rx={6} fill="#1a0b0f" stroke={C.yel} strokeWidth={1.5} />
              <T x={388} y={196} s={8} a="middle">
                {L("Пароль Mac", "Mac password")}
              </T>
              <T x={388} y={212} s={10} a="middle" c={C.iosMute}>
                ● ● ● ● ● ●
              </T>
              <Num x={318} y={176} n={3} />
            </Art>
          ),
        }
      : {
          title: L("Первый запуск на Windows", "First launch on Windows"),
          text: L(
            "Если появится синее окно SmartScreen — «Подробнее» → «Выполнить в любом случае». Если компьютеру не хватает службы Apple для связи с iPhone, установщик скачает её сам, а Windows спросит разрешение — «Да».",
            "If the blue SmartScreen window appears — “More info” → “Run anyway”. If the PC lacks the Apple service for the iPhone, the installer downloads it by itself and Windows asks for permission — “Yes”.",
          ),
          art: (
            <Art>
              <Win x={10} y={10} w={244} h={230} fill="#0b3f78" />
              <T x={24} y={52} s={11} w={700}>
                {L("Windows защитил ваш", "Windows protected")}
              </T>
              <T x={24} y={67} s={11} w={700}>
                {L("компьютер", "your PC")}
              </T>
              <T x={24} y={88} s={7.5} c="#cfe0f5">
                {L("Фильтр SmartScreen предотвратил", "Microsoft Defender SmartScreen")}
              </T>
              <T x={24} y={99} s={7.5} c="#cfe0f5">
                {L("запуск неопознанного приложения.", "prevented an unrecognized app.")}
              </T>
              <rect x={20} y={110} width={62} height={18} rx={3} fill="none" stroke={C.yel} strokeWidth={2} />
              <T x={26} y={122} s={8} c="#ffffff" w={700}>
                {L("Подробнее", "More info")}
              </T>
              <Num x={20} y={110} n={1} />
              <Btn x={112} y={206} w={134} label={L("Выполнить в любом случае", "Run anyway")} hi />
              <Num x={108} y={216} n={2} />
              <Arrow d="M258 150 C 268 150, 268 150, 276 150" />
              <Win x={280} y={10} w={230} h={230} fill="#1f1f1f" title={L("Контроль учётных записей", "User Account Control")} />
              <T x={292} y={48} s={8.5} w={700}>
                {L("Разрешить этому приложению", "Do you want to allow this app")}
              </T>
              <T x={292} y={60} s={8.5} w={700}>
                {L("вносить изменения на вашем", "to make changes to your")}
              </T>
              <T x={292} y={72} s={8.5} w={700}>
                {L("устройстве?", "device?")}
              </T>
              <T x={292} y={98} s={8.5} w={700}>
                Apple Mobile Device Support
              </T>
              <T x={292} y={112} s={7.5} c={C.iosMute}>
                {L("Проверенный издатель: Apple Inc.", "Verified publisher: Apple Inc.")}
              </T>
              <Btn x={292} y={206} w={100} label={L("Да", "Yes")} hi blue />
              <Btn x={400} y={206} w={100} label={L("Нет", "No")} />
              <Num x={288} y={216} n={3} />
            </Art>
          ),
        };

  return [
    first,
    {
      title: L("Подключите iPhone кабелем", "Connect the iPhone with a cable"),
      text: L(
        "Разблокируйте iPhone и нажмите «Доверять», затем введите код-пароль iPhone. Кабель нужен только в первый раз — дальше установщик сам видит iPhone по Wi-Fi.",
        "Unlock the iPhone and tap “Trust”, then enter the iPhone passcode. The cable is needed only the first time — after that the installer sees the iPhone over Wi-Fi.",
      ),
      art: (
        <Art>
          <Win x={14} y={40} w={230} h={140} title="CubeXCraft Installer" />
          <T x={30} y={84} s={10} w={800} c="#f7eaea" f={MONO}>
            CUBE<tspan fill={C.red}>X</tspan>CRAFT
          </T>
          <path d="M0 196 L14 180 H244 L258 196 Z" fill="#1a0b0f" stroke={C.line} />
          <path d="M250 188 C 300 200, 320 250, 405 238" fill="none" stroke="#c4a8ac" strokeWidth={3} strokeDasharray="1 0" />
          <T x={292} y={234} s={8} c="#c4a8ac" f={MONO}>
            USB
          </T>
          <Phone x={330} title={L("iPhone", "iPhone")} />
          <Alert
            x={338}
            y={64}
            lines={[L("Доверять этому", "Trust This"), L("компьютеру?", "Computer?")]}
            sub={[L("Настройки и данные будут", "Your settings and data will"), L("доступны с компьютера", "be accessible from it")]}
            buttons={[L("Доверять", "Trust"), L("Не доверять", "Don't Trust")]}
          />
          <Num x={334} y={131} n={1} />
          <T x={405} y={190} s={8} a="middle">
            {L("Код-пароль iPhone", "iPhone passcode")}
          </T>
          <T x={405} y={206} s={10} a="middle" c={C.iosMute}>
            ● ● ● ● ● ●
          </T>
          <Num x={350} y={196} n={2} />
        </Art>
      ),
    },
    {
      title: L("Войдите в Apple ID (блок 01)", "Sign in with Apple ID (block 01)"),
      text: L(
        "Почта и пароль Apple ID — подойдёт обычный бесплатный, пароль уходит только в Apple. Потом введите код, который придёт на iPhone.",
        "Your Apple ID email and password — a regular free one works, the password goes only to Apple. Then enter the code that comes to your iPhone.",
      ),
      art: (
        <Art>
          <Installer x={10} y={8} w={310} h={234} />
          <Kick x={24} y={66} w={284} label="01 · APPLE ID" />
          <rect x={20} y={74} width={290} height={150} fill="none" stroke={C.yel} strokeWidth={1.5} strokeDasharray="4 3" />
          <rect x={32} y={84} width={266} height={22} fill="#14060a" stroke={C.line} />
          <T x={40} y={98} s={8} c={C.mute}>
            {L("Почта Apple ID", "Apple ID email")}
          </T>
          <rect x={32} y={112} width={266} height={22} fill="#14060a" stroke={C.line} />
          <T x={40} y={126} s={8} c={C.mute}>
            {L("Пароль", "Password")}
          </T>
          <Btn x={32} y={142} w={120} h={22} label={L("ВОЙТИ", "SIGN IN")} red hi />
          <Num x={28} y={153} n={1} />
          <rect x={32} y={184} width={160} height={24} fill="#14060a" stroke={C.yel} strokeWidth={1.5} />
          <T x={40} y={200} s={9} c={C.iosMute}>
            ● ● ●  ● ● ●
          </T>
          <T x={200} y={200} s={8} c={C.mute}>
            {L("← код с iPhone", "← code from iPhone")}
          </T>
          <Num x={28} y={196} n={3} />
          <Phone x={352} y={20} w={156} h={214} />
          <Alert
            x={362}
            y={70}
            w={136}
            lines={[L("Код проверки", "Verification code")]}
            sub={[L("появится на iPhone —", "appears on the iPhone —"), L("введите его в установщике", "enter it in the installer")]}
            buttons={[L("Готово", "Done")]}
            hi={-1}
          />
          <Num x={362} y={70} n={2} />
          <T x={430} y={172} s={11} a="middle" c="#ffffff" w={700}>
            ● ● ●  ● ● ●
          </T>
          <Arrow d="M372 178 C 330 200, 304 198, 284 196" />
        </Art>
      ),
    },
    {
      title: L("Выберите iPhone и установите (блоки 02 и 03)", "Pick the iPhone and install (blocks 02 and 03)"),
      text: L(
        "В блоке 02 выберите iPhone, в блоке 03 нажмите «Установить CubeXCraft». Установщик сам скачает игру, подпишет и поставит её вместе со StikDebug — на экране видны проценты.",
        "Pick the iPhone in block 02 and click “Install CubeXCraft” in block 03. The installer downloads, signs and installs the game together with StikDebug — progress is shown on screen.",
      ),
      art: (
        <Art>
          <Installer x={50} y={8} w={420} h={234} />
          <Kick x={66} y={64} w={390} label="02 · IPHONE" />
          <rect x={66} y={72} width={388} height={28} fill="#14060a" stroke={C.yel} strokeWidth={2} />
          <T x={78} y={90} s={9} w={700}>
            iPhone
          </T>
          <rect x={372} y={79} width={74} height={14} fill="#00ff9f10" stroke="#0a5c3d" />
          <T x={409} y={89} s={7} c={C.ok} a="middle" f={MONO}>
            {L("ВЫБРАН", "SELECTED")}
          </T>
          <Num x={62} y={86} n={1} />
          <Kick x={66} y={124} w={390} label={L("03 · УСТАНОВКА", "03 · INSTALL")} yellow />
          <Btn x={66} y={132} w={388} h={32} label={L("УСТАНОВИТЬ CUBEXCRAFT", "INSTALL CUBEXCRAFT")} red hi />
          <Num x={62} y={148} n={2} />
          <T x={78} y={190} s={8} c={C.ok}>
            {L("✓ Скачивание свежего CubeXCraft", "✓ Downloading the latest CubeXCraft")}
          </T>
          <T x={78} y={206} s={8}>
            {L("◌ Подпись и установка CubeXCraft", "◌ Signing and installing CubeXCraft")}
          </T>
          <T x={92} y={220} s={7.5} c="#c4a8ac" f={MONO}>
            {L("Передача на iPhone: … %", "Sending to iPhone: … %")}
          </T>
        </Art>
      ),
    },
    {
      title: L("Поставьте LocalDevVPN по QR-коду", "Install LocalDevVPN via the QR code"),
      text: L(
        "Когда в установщике появится QR-код, наведите на него камеру iPhone и нажмите «Загрузить» в App Store (бесплатно). Установщик сам заметит LocalDevVPN и закончит настройку.",
        "When a QR code appears in the installer, point the iPhone camera at it and tap “Get” in the App Store (free). The installer notices LocalDevVPN and finishes the setup.",
      ),
      art: (
        <Art>
          <Win x={6} y={34} w={176} h={190} title="CubeXCraft Installer" />
          <T x={94} y={70} s={8} a="middle" c="#c4a8ac">
            LocalDevVPN · App Store
          </T>
          <QR x={58} y={84} size={72} />
          <Num x={50} y={84} n={1} />
          <Arrow d="M186 128 C 196 128, 198 128, 206 128" />
          <Phone x={210} y={8} w={140} title={L("Камера", "Camera")} />
          <path d="M240 82 h-12 v12 M310 82 h12 v12 M240 170 h-12 v-12 M310 170 h12 v-12" fill="none" stroke={C.yel} strokeWidth={2.5} />
          <QR x={244} y={94} size={60} />
          <rect x={222} y={186} width={116} height={20} rx={10} fill="#ffffff22" />
          <T x={280} y={199} s={7.5} a="middle">
            {L("Открыть в App Store", "Open in App Store")}
          </T>
          <Arrow d="M354 128 C 362 128, 364 128, 372 128" />
          <Phone x={376} y={8} w={140} title="App Store" />
          <rect x={388} y={64} width={42} height={42} rx={10} fill="#1c3f8f" />
          <T x={409} y={91} s={14} a="middle" w={800}>
            ⛨
          </T>
          <T x={438} y={80} s={9} w={700}>
            LocalDevVPN
          </T>
          <rect x={438} y={88} width={58} height={20} rx={10} fill={C.iosBlue} stroke={C.yel} strokeWidth={2} />
          <T x={467} y={101.5} s={8} a="middle" w={700}>
            {L("Загрузить", "Get")}
          </T>
          <Num x={500} y={98} n={2} />
        </Art>
      ),
    },
    {
      title: L("Доверьте свой Apple ID на iPhone", "Trust your Apple ID on the iPhone"),
      text: L(
        "Настройки → Основные → VPN и управление устройством → ваш Apple ID → «Доверять». Без этого iPhone не запустит CubeXCraft.",
        "Settings → General → VPN & Device Management → your Apple ID → “Trust”. Without this the iPhone won't launch CubeXCraft.",
      ),
      art: (
        <Art>
          <Phone x={0} title={L("Настройки", "Settings")} />
          <Row x={8} y={60} label={L("Уведомления", "Notifications")} />
          <Row x={8} y={82} label={L("Звуки, тактильные сигналы", "Sounds & Haptics")} />
          <Row x={8} y={110} label={L("Основные", "General")} hi />
          <Row x={8} y={132} label={L("Экран и яркость", "Display & Brightness")} />
          <Num x={8} y={120} n={1} />
          <Arrow d="M152 120 C 164 120, 168 120, 180 120" />
          <Phone x={185} title={L("Основные", "General")} />
          <Row x={193} y={60} label={L("Об этом устройстве", "About")} />
          <Row x={193} y={82} label={L("Обновление ПО", "Software Update")} />
          <Row x={193} y={104} label={L("Хранилище iPhone", "iPhone Storage")} />
          <Row x={193} y={140} label={L("VPN и управление устр-вом", "VPN & Device Management")} hi />
          <Num x={193} y={150} n={2} />
          <Arrow d="M337 150 C 350 150, 352 140, 365 140" />
          <Phone x={370} title={L("VPN и управление", "VPN & Device")} />
          <T x={382} y={64} s={6.5} c={C.iosMute} f={MONO}>
            {L("ПРИЛОЖЕНИЕ РАЗРАБОТЧИКА", "DEVELOPER APP")}
          </T>
          <Row x={378} y={70} label={L("Ваш Apple ID", "Your Apple ID")} hi />
          <Num x={378} y={80} n={3} />
          <Alert
            x={378}
            y={112}
            lines={[L("Доверять разработчику?", "Trust this developer?")]}
            sub={[L("приложения с вашим Apple ID", "apps signed with your Apple ID")]}
            buttons={[L("Доверять", "Trust"), L("Отменить", "Cancel")]}
          />
          <Num x={378} y={163} n={4} />
        </Art>
      ),
    },
    {
      title: L("Включите режим разработчика", "Turn on Developer Mode"),
      text: L(
        "Настройки → Конфиденциальность и безопасность → в самом низу «Режим разработчика» → Вкл. → «Перезагрузить». После перезагрузки разблокируйте iPhone, нажмите «Включить» и введите код-пароль. Пункт появляется только после установки (шаг 4).",
        "Settings → Privacy & Security → at the very bottom “Developer Mode” → On → “Restart”. After the restart unlock the iPhone, tap “Turn On” and enter the passcode. The item appears only after installing (step 4).",
      ),
      art: (
        <Art>
          <Phone x={0} title={L("Конфиденциальность", "Privacy &")} title2={L("и безопасность", "Security")} />
          <Row x={8} y={64} label={L("Службы геолокации", "Location Services")} />
          <Row x={8} y={86} label={L("Отслеживание", "Tracking")} />
          <T x={75} y={124} s={10} c={C.iosMute} a="middle">
            ⋮
          </T>
          <T x={14} y={150} s={6.5} c={C.iosMute} f={MONO}>
            {L("БЕЗОПАСНОСТЬ · в самом низу", "SECURITY · at the very bottom")}
          </T>
          <Row x={8} y={156} label={L("Режим разработчика", "Developer Mode")} value={L("Выкл.", "Off")} hi />
          <Num x={8} y={166} n={1} />
          <Arrow d="M152 166 C 164 166, 168 140, 180 140" />
          <Phone x={185} title={L("Режим разработчика", "Developer Mode")} />
          <Row x={193} y={60} label={L("Режим разработчика", "Developer Mode")} toggle hi />
          <Num x={193} y={70} n={2} />
          <Alert
            x={193}
            y={100}
            lines={[L("Нужна перезагрузка", "Restart required")]}
            sub={[L("чтобы включить режим", "to turn on Developer Mode")]}
            buttons={[L("Перезагрузить", "Restart"), L("Отменить", "Cancel")]}
          />
          <Num x={193} y={151} n={3} />
          <Arrow d="M337 150 C 350 150, 352 130, 365 130" />
          <Phone x={370} title={L("После перезагрузки", "After restart")} />
          <Alert
            x={378}
            y={70}
            lines={[L("Включить режим", "Turn on Developer"), L("разработчика?", "Mode?")]}
            buttons={[L("Включить", "Turn On"), L("Отменить", "Cancel")]}
          />
          <Num x={378} y={116} n={4} />
          <T x={445} y={190} s={8} a="middle">
            {L("Код-пароль iPhone", "iPhone passcode")}
          </T>
          <T x={445} y={206} s={10} a="middle" c={C.iosMute}>
            ● ● ● ● ● ●
          </T>
          <Num x={390} y={196} n={5} />
        </Art>
      ),
    },
    {
      title: L("Откройте CubeXCraft и нажмите «Играть»", "Open CubeXCraft and tap “Play”"),
      text: L(
        "Войдите аккаунтом CubeXCraft. В первый раз iPhone спросит про VPN — «Разрешить». StikDebug откроется на секунду и сам вернёт в игру — JIT включается сам.",
        "Sign in with your CubeXCraft account. The first time the iPhone asks about VPN — “Allow”. StikDebug opens for a second and returns you to the game — JIT turns on by itself.",
      ),
      art: (
        <Art>
          <Phone x={0} bg="#12060a" />
          <path d="M75 52 l22 12 v24 l-22 12 l-22 -12 v-24 z" fill="#2a0810" stroke={C.red} strokeWidth={2} />
          <path d="M75 62 l12 7 l-12 7 l-12 -7 z" fill={C.yel} />
          <T x={75} y={124} s={10} w={800} a="middle" f={MONO} c="#f7eaea">
            CUBE<tspan fill={C.red}>X</tspan>CRAFT
          </T>
          <Btn x={20} y={170} w={110} h={28} label={L("ИГРАТЬ", "PLAY")} red hi />
          <Num x={16} y={184} n={1} />
          <Arrow d="M152 150 C 164 150, 168 150, 180 150" />
          <Phone x={185} bg="#12060a" />
          <Alert
            x={193}
            y={62}
            lines={[L("«LocalDevVPN» хочет", "“LocalDevVPN” Would"), L("добавить конфигурации", "Like to Add VPN"), L("VPN", "Configurations")]}
            buttons={[L("Разрешить", "Allow"), L("Не разрешать", "Don't Allow")]}
          />
          <Num x={193} y={124} n={2} />
          <Arrow d="M337 150 C 350 150, 352 150, 365 150" />
          <Phone x={370} />
          <rect x={410} y={70} width={70} height={70} rx={16} fill="#0a84ff" />
          <T x={445} y={116} s={30} a="middle" w={800}>
            ⚡
          </T>
          <T x={445} y={162} s={8.5} a="middle" w={700}>
            StikDebug
          </T>
          <T x={445} y={180} s={7.5} a="middle" c={C.iosMute}>
            {L("на секунду — и", "for a second — and")}
          </T>
          <T x={445} y={192} s={7.5} a="middle" c={C.iosMute}>
            {L("обратно в игру", "back to the game")}
          </T>
        </Art>
      ),
    },
  ];
}

export const HowTo = ({
  isOpen,
  close,
  lang,
  platform,
}: {
  isOpen: boolean;
  close: () => void;
  lang: Lang;
  platform: Platform;
}) => {
  const [index, setIndex] = useState(0);
  const list = steps(lang, platform);
  const step = list[Math.min(index, list.length - 1)];
  const ru = lang === "ru";
  return (
    <Modal isOpen={isOpen} close={close} zIndex={2500}>
      <div className="cxh">
        <div className="cxh-top">
          <span>{ru ? "Как установить" : "How to install"}</span>
          <span>
            {ru ? "Шаг" : "Step"} {index + 1} / {list.length}
          </span>
        </div>
        {step.art}
        <h3>{step.title}</h3>
        <p>{step.text}</p>
        <div className="cxh-nav">
          <button disabled={index === 0} onClick={() => setIndex(index - 1)}>
            {ru ? "Назад" : "Back"}
          </button>
          <div className="cxh-dots">
            {list.map((_, k) => (
              <span key={k} className={k === index ? "on" : ""} onClick={() => setIndex(k)} />
            ))}
          </div>
          {index < list.length - 1 ? (
            <button className="cxc-install" onClick={() => setIndex(index + 1)}>
              {ru ? "Далее" : "Next"}
            </button>
          ) : (
            <button className="cxc-install" onClick={close}>
              {ru ? "Понятно" : "Got it"}
            </button>
          )}
        </div>
      </div>
    </Modal>
  );
};
