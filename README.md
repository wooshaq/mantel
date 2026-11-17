# Mantel WsQ – wydania

Zbudowane pliki exe z gałęzi `main` tego repo .

## 0.6.0-alpha.1 (2026-11-17) – natywny, wersja testowa

Kod: commit `ba5c0ab` na `main` (folder `native/`). Pobierz: [portable](https://github.com/wooshaq/mantel/raw/wydania/0.6.0-alpha.1/MantelNative_0.6.0-alpha.1_x64-portable.exe)

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

## 0.5.4-wsq.3 (2026-11-17) – aktualna

Kod: commit `47fe7d3` na `main`. Pobierz: [instalator](https://github.com/wooshaq/mantel/raw/wydania/0.5.4-wsq.3/Mantel_0.5.4-wsq.3_x64-setup.exe) · [portable](https://github.com/wooshaq/mantel/raw/wydania/0.5.4-wsq.3/Mantel_0.5.4-wsq.3_x64-portable.exe)

Mantel 0.5.4 WsQ, wersja 3: **język polski** i poprawka konfiguracji. Zawiera wszystko z wsq.2 (brak PowerShella, aktualizatora i Trybu dewelopera).

**Nowe**
- Polski interfejs, z poprawną odmianą liczebników („1 gra”, „3 gry”, „5 gier”). Na polskim Windowsie włącza się sam, a sklep Steam w Mantelu też pokazuje się po polsku.

**Poprawione**
- Konfiguracja: gdy Windows był w języku, którego Mantel nie miał (np. polskim), wybranie „English” zostawiało następny krok pusty. Teraz krok wyświetla się od razu.
- Usunięta nieużywana wtyczka okien dialogowych (`tauri-plugin-dialog`) razem z jej uprawnieniem. Program ma przez to mniej zbędnego kodu.

**Pliki**
- `Mantel_0.5.4-wsq.3_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień admina.
- `Mantel_0.5.4-wsq.3_x64-portable.exe`: sam program, bez instalacji (wymaga WebView2, który jest w Windows 11).

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie: wybierz *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla nowego, niepodpisanego instalatora NSIS – [raport](https://www.virustotal.com/gui/file/ee5f12d2cfbb77d7f393e734bfd2c07ba3bc2cafaa369c96c689445e7e73dd9a)
- portable: 1/59 (Trapmine, heurystyka ML) – [raport](https://www.virustotal.com/gui/file/3e49a10a73f2bd2d35a5ba3e59e4ff678b68bd00b625c1bffdc24379c3ba0c76)

**SHA-256**
```
ee5f12d2cfbb77d7f393e734bfd2c07ba3bc2cafaa369c96c689445e7e73dd9a  Mantel_0.5.4-wsq.3_x64-setup.exe
3e49a10a73f2bd2d35a5ba3e59e4ff678b68bd00b625c1bffdc24379c3ba0c76  Mantel_0.5.4-wsq.3_x64-portable.exe
```

## 0.5.4-wsq.2 (2026-11-17)

Kod: commit `6aa427d` na `main`. Pobierz: [instalator](https://github.com/wooshaq/mantel/raw/wydania/0.5.4-wsq.2/Mantel_0.5.4-wsq.2_x64-setup.exe) · [portable](https://github.com/wooshaq/mantel/raw/wydania/0.5.4-wsq.2/Mantel_0.5.4-wsq.2_x64-portable.exe)

Mantel 0.5.4 WsQ, wersja 2: **bez PowerShella**. Zastępuje 0.5.4-wsq.1, której nie używaj, bo Bitdefender ją blokował.

**Dlaczego**
Wersja wsq.1 (jak oryginał) przy starcie uruchamiała w tle ukryte procesy PowerShell (`-ExecutionPolicy Bypass`, `-EncodedCommand`), żeby znaleźć gry Xbox i skróty z menu Start. Skrypty tylko czytały dane, ale Bitdefender (Zaawansowana Ochrona) słusznie uznał takie zachowanie za podejrzane i zablokował program.

**Co zmieniono**
- Gry Xbox / Game Pass są wyszukiwane przez systemowy PackageManager i pliki AppxManifest.xml, a nie przez `Get-AppxPackage`.
- Aplikacje z menu Start są odczytywane z `shell:AppsFolder` i skrótów (IShellLink), a nie przez `Get-StartApps` / `WScript.Shell`.
- Nazwa gry z pliku exe jest odczytywana z jego zasobów wersji (GetFileVersionInfo).
- Gry Game Pass / Sklepu MS: przycisk Zainstaluj otwiera stronę gry w aplikacji Xbox, gdzie ją instalujesz. Odinstalowanie otwiera Ustawienia Windows → Aplikacje. Gry Steam instalują się i odinstalowują jak dotąd.
- Program w ogóle nie zawiera PowerShella (sprawdzone w samym exe).
- Nadal obowiązują zmiany z wsq.1: brak auto-aktualizatora, brak Trybu dewelopera i ustawiania jako ekran główny, brak skryptów `tools/`.

**Pliki**
- `Mantel_0.5.4-wsq.2_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień admina.
- `Mantel_0.5.4-wsq.2_x64-portable.exe`: sam program, bez instalacji (wymaga WebView2, który jest w Windows 11).

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie: wybierz *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla nowego, niepodpisanego instalatora NSIS – [raport](https://www.virustotal.com/gui/file/b240038c7755ee00b9ae6102dfa64ccb15e7f1343b021016d4992a9e96291a0a)
- portable: 1/71 (Trapmine, heurystyka ML) – [raport](https://www.virustotal.com/gui/file/abf21f89251b88dc92c50c06cb861bbfd55f566eb0a21a98f9354978357d216f)

**SHA-256**
```
b240038c7755ee00b9ae6102dfa64ccb15e7f1343b021016d4992a9e96291a0a  Mantel_0.5.4-wsq.2_x64-setup.exe
abf21f89251b88dc92c50c06cb861bbfd55f566eb0a21a98f9354978357d216f  Mantel_0.5.4-wsq.2_x64-portable.exe
```


## 0.5.4-wsq.1 (2026-11-17) – wycofana: blokowana przez Bitdefender (ukryty PowerShell), użyj wsq.2

Pobierz: [instalator](https://github.com/wooshaq/mantel/raw/wydania/0.5.4-wsq.1/Mantel_0.5.4-wsq.1_x64-setup.exe) · [portable](https://github.com/wooshaq/mantel/raw/wydania/0.5.4-wsq.1/Mantel_0.5.4-wsq.1_x64-portable.exe)

Mantel 0.5.4 z poprawkami bezpieczeństwa WsQ, na bazie jarto666/mantel v0.5.4.

**Co zmieniono względem oryginału**
- Usunięty auto-aktualizator: program sam nic nie pobiera ani nie instaluje (wcześniej ściągał wydania z repo autora).
- Mantel nigdy nie prosi o uprawnienia administratora i nie włącza Trybu dewelopera Windows.
- Usunięte ustawianie Mantela jako ekranu głównego trybu Xbox: instalator nie uruchamia PowerShella, nie rejestruje paczki MSIX i nie zapisuje nic w rejestrze.
- Usunięte skrypty deweloperskie autora (`tools/`: SSH, `irm | iex`).

**Pliki**
- `Mantel_0.5.4-wsq.1_x64-setup.exe`: instalator (dla bieżącego użytkownika, bez admina).
- `Mantel_0.5.4-wsq.1_x64-portable.exe`: sam program bez instalacji (wymaga WebView2, który jest w Windows 11).

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać „System Windows ochronił komputer”: *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine: heurystyki ML, typowe dla nowego niepodpisanego instalatora NSIS) – [raport](https://www.virustotal.com/gui/file/072ac9e3cf714b57ca85404cea417d1851bb3aacc73faea79642d16fdff2859d)
- portable: 1/67 (Trapmine: heurystyka ML) – [raport](https://www.virustotal.com/gui/file/81f9a0b33d01d9bff0a37a619769f31f512987b1b339a689d5275b789243628e)

**SHA-256**
```
072ac9e3cf714b57ca85404cea417d1851bb3aacc73faea79642d16fdff2859d  Mantel_0.5.4-wsq.1_x64-setup.exe
81f9a0b33d01d9bff0a37a619769f31f512987b1b339a689d5275b789243628e  Mantel_0.5.4-wsq.1_x64-portable.exe
```
