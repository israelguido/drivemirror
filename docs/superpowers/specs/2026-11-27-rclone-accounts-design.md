# DriveMirror Rclone Accounts Design

## Intent

DriveMirror must let a desktop user configure Google Drive without leaving the app for rclone's terminal wizard. Rclone remains the owner of OAuth, tokens, and its configuration file.

## Behavior

- Add an “Rclone accounts” page that lists configured remotes and refreshes after changes.
- Create a Google Drive remote from a validated remote name, Google Client ID, and masked Client Secret.
- Drive rclone's documented `config create --non-interactive` continuation protocol and show its questions and errors in the window.
- Let rclone open the system browser for Google authorization. The app shows progress while waiting and never handles the Google password or authorization code directly.
- Support reconnecting an existing remote and removing one only after an explicit confirmation.
- Link to the official Google Client ID instructions. Creating Google Cloud projects, OAuth credentials, or account permissions is outside the app.
- Keep Client Secret values only in widget/process memory long enough to start rclone. Never persist or log them; rclone alone writes its own configuration.

## Approach

Use `QProcess`-equivalent Rust process spawning with literal arguments and parse rclone's non-interactive JSON question/result protocol. This was selected over embedding OAuth or shelling out to an interactive terminal because it preserves rclone's security boundary and provides a native GUI.

## Constraints

- No shell command strings.
- Only Google Drive remote creation is required in this MVP; existing remotes of other types may be listed but not edited.
- Missing rclone, malformed JSON, browser cancellation, and authentication failure remain visible and retryable.
- Remote deletion changes rclone configuration only; it never deletes files from Google Drive.
