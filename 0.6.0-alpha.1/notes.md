**Mantel natywny (bez WebView2): pierwszy etap.** To wersja testowa. Stabilna jest dalej [0.5.4-wsq.3](https://github.com/wooshaq/mantel/releases/tag/v0.5.4-wsq.3).

Ten sam Mantel, ale interfejs jest napisany natywnie w Slint zamiast przeglądarki Edge (WebView2). Backend w Ruście jest ten sam, więc biblioteka, czas gry, ulubione i ustawienia przechodzą 1:1. Program korzysta z tego samego folderu danych co wsq.3.

**Co już działa**
- Ekran główny: tło i logo gry, czas gry, ostatnie uruchomienie, osiągnięcia, rozmiar na dysku, przyciski Graj i ulubione, półka okładek.
- Biblioteka: siatka wszystkich zainstalowanych gier.
- Uruchamianie gier i śledzenie sesji, także dla gier odpalonych poza Mantelem, gdy Mantel działa.
- Pad (XInput, z powtarzaniem kierunków) i klawiatura: strzałki, Enter, Esc, F (ulubiona), Q/E (zakładki), F5 (skanuj), F11 (pełny ekran).
- Wszystkie 6 języków, w tym polski.

**Czego jeszcze nie ma (kolejne etapy)**
Strony gry ze szczegółami, ustawień, zakładki Aplikacje, wyszukiwarki, centrum sterowania, sklepów Steam i Game Pass, logowania do kont oraz trybu w zasobniku. Potem dojdą: osiągnięcia z publicznego profilu Steam, Big Picture i HDR.

**Plik**
- `MantelNative_0.6.0-alpha.1_x64-portable.exe`: sam program, bez instalacji, 14 MB. Nie potrzebuje WebView2.

W moim środowisku testowym zajmował około 40 MB RAM w 1080p (renderowanie programowe). Na Windowsie domyślnie rysuje przez OpenGL na karcie graficznej. Nie testowałem go na prawdziwym Windowsie, tylko na Linuksie.

Plik nie jest podpisany cyfrowo, więc SmartScreen może pokazać ostrzeżenie: wybierz *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17):** 0/70 – czysto ([raport](https://www.virustotal.com/gui/file/9b32e9b524ab51a90ff8ed45265d0ca5619583f77397caeeb688c33d739c5fe4))

**SHA-256**
```
9b32e9b524ab51a90ff8ed45265d0ca5619583f77397caeeb688c33d739c5fe4  MantelNative_0.6.0-alpha.1_x64-portable.exe
```
