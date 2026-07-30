# Prowler

![Prowler](https://i.ibb.co/vSLJDRZ/prowler.png)

Prowler is a lightweight desktop tool for League of Legends built with Tauri and React, designed to enhance your experience in League of Legends client. Heavily inspired by: [Tiamat](https://github.com/369gabriel/tiamat)

> [!WARNING]
> **Use at your own risk.** Prowler is not affiliated with, endorsed, or sponsored by Riot Games. Using third-party tools may violate Riot's Terms of Service and could result in penalties, including account bans. You are solely responsible for any consequences of using this software.

## Features

- **Account Switcher**: Securely save multiple Riot account sessions and switch between them with a single click. No more typing passwords.

- **Automation**: Automate repetitive pre-game tasks.
  - **Instalock & Prepick**: Automatically lock in or hover your desired champion based on your assigned role. Includes priority lists and a "random" option.
  - **Autoban**: Ban a champion from a per-role priority list, intelligently skipping champions hovered by teammates.
  - **Auto Accept**: Instantly accept the ready check when a match is found.
  - **Auto Runes**: Automatically applies the recommended rune page for your champion and role.
  - **Auto Summoners**: Sets your summoner spells based on your assigned role.

- **Profile Customization**: Modify your in-client presence.
  - **Client-Side Icon**: Set any champion or summoner icon as your chat icon (visible only to you).
  - **Profile Background**: Use any skin splash art as your profile background, regardless of ownership.
  - **Status Message**: Set a custom status message that appears in your friends' lists.
  - **Profile Badges**: Glitch or clear the challenge badges on your profile.

- **Custom Rune Pages**: A complete rune page editor. Build, save, and apply your own rune pages on demand.

- **Game Tools**:
  - **Dodge**: Instantly quit a champ select lobby.
  - **Restart UX**: Resets the client's user interface to fix visual glitches or freezes without a full relog.

- **Social Tools**:
  - **Appear Offline**: Disconnect from the chat service to appear offline to your friends while still being able to play.
  - **Friend List Management**: Quickly count or remove all friends from your list.

## Installation

1.  Go to the [**Releases**](https://github.com/whosneksio/prowler/releases) page.
2.  Download the `.msi` installer for the latest version.
3.  Run the installer and follow the on-screen instructions.

### Verifying releases

Every release is built by [GitHub Actions](.github/workflows/release.yml) from the public source code, with cryptographic build provenance. To verify that a downloaded installer was built from this repository (requires the [GitHub CLI](https://cli.github.com/)):

```sh
gh attestation verify Prowler_X.Y.Z_x64_en-US.msi --repo whosneksio/prowler
```

A successful verification names this repository, the release workflow, and the exact commit the installer was built from. All attestations are also listed at [github.com/whosneksio/prowler/attestations](https://github.com/whosneksio/prowler/attestations).

To use the Account Switcher:
1.  Log in to the Riot Client with the **“Stay signed in”** option checked.
2.  In Prowler, navigate to the **Accounts** view.
3.  Click **"Save current session"**.
4.  Repeat for each account you want to save.

## Building from Source

To build Prowler yourself, you'll need to have [Rust](https://www.rust-lang.org/tools/install) and [Bun](https://bun.sh/) installed.

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/whosneksio/prowler.git
    cd prowler
    ```

2.  **Install frontend dependencies:**
    ```sh
    bun install
    ```

3.  **Run in development mode:**
    The application will launch with hot-reloading for the frontend.
    ```sh
    bun run tauri dev
    ```

4.  **Build the application:**
    This will create a production-ready executable and installer in `src-tauri/target/release/bundle/`.
    ```sh
    bun run tauri build
    ```