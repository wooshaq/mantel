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
