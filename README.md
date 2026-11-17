# Mantel WsQ – wydania

Zbudowane pliki exe z gałęzi `main` tego repo (commit `0286484`).

## 0.5.4-wsq.1 (2026-11-17)

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
