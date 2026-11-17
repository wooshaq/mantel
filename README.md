# Mantel WsQ – wydania

Zbudowane pliki exe z gałęzi `main` tego repo .

## 0.5.4-wsq.2 (2026-11-17) – aktualna

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
