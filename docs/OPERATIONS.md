# weathr operations runbook

This runbook describes the safe operating contract for a local watchdog that keeps one foreground `weathr` TUI available in a dedicated `tmux` pane. It intentionally uses abstract target names. Do not add hostnames, IP addresses, process IDs, SSH material, or other private deployment data here.

## Scope and purpose

The watchdog is a narrow availability check for the managed `weathr` terminal. Its job is to notice that the expected TUI has exited or that its terminal state is no longer valid, then—only after the checks below—start the exact approved `weathr` process in the same managed target.

It is **not** a general process supervisor. It must not restart a shell, a `tmux` server, a web service, an SSH service, or any process other than the exact managed `weathr` process.

## Abstract target structure

The deployment has one explicitly configured target. The names are deployment-local and must not be copied into public documentation:

```text
tmux server
└── managed session
    └── managed window
        └── managed pane
            └── foreground `weathr`
```

The watchdog must resolve and inspect that one session/window/pane, rather than scanning all sessions or matching the first pane that happens to contain a similar string. A target mismatch, duplicate match, missing target, or permission error is an unsafe state and must fail closed.

## Monitoring contract

- Check the target every **15 seconds**.
- Inspect both the pane state and the exact foreground process; do not treat a frozen or empty capture as proof that another process may be started.
- Recognize these terminal states explicitly:
  - **q/quit state:** `weathr` accepts `q` or `Q` to exit. A visible `Press 'q' to quit` hint is normal HUD text, not an error. The hint is hidden in the default compact HUD and appears when HUD details are expanded with F1, so its absence is not evidence that `weathr` is down. A pane transition after a deliberate quit must be distinguished from an unexpected exit.
  - **Prompt state:** a shell or command prompt means the TUI is no longer the foreground application. Use the deployment's approved prompt recognition; never use a broad pattern that could match application output.
  - **Alternate-screen state:** a healthy renderer occupies the terminal alternate screen. Leaving that screen is an explicit signal that the TUI has exited or cleaned up. If alternate-screen state cannot be determined reliably, do not recover automatically.
- Keep the q/quit, prompt, and alternate-screen signals separate. A single captured line or a single missing marker is not enough to identify the cause.

The TUI enters the alternate screen and raw input mode during startup, and its normal cleanup leaves the alternate screen, restores the cursor/color state, and disables raw mode. These transitions are expected during a deliberate exit and must not be mistaken for permission to restart an unrelated command.

## Double revalidation before recovery

A recovery action requires two consecutive, independent validations of the same target:

1. The scheduled 15-second check records an unhealthy or exited state and the exact target identity.
2. A second check repeats the target, process, prompt, and alternate-screen observations. It must reach the same conclusion; if the state becomes healthy or ambiguous, clear the candidate recovery and take no action.

Do not start a process during the first observation. Revalidate immediately before any manual or automated recovery as well, because a user may have pressed `q`, Ctrl+C, or F1 between checks. Record only local operational evidence; do not place pane captures containing private data in the repository.

## Fail-closed behavior

When any required fact is uncertain, the safe action is **no action**. This includes:

- the configured tmux target cannot be resolved exactly;
- more than one session, window, pane, or process matches;
- the process command line is truncated or does not identify exactly `weathr`;
- prompt or alternate-screen detection is ambiguous;
- tmux capture, permissions, or process inspection fails;
- the pane is being manually operated or its state changes during revalidation.

Log the reason locally for an operator, leave all processes and services untouched, and request manual inspection. Never use `killall`, broad `pkill`, a wildcard process match, or a service-manager restart as a fallback.

## Allowed recovery action

After both validations agree that the managed application has exited, the only automatic start permitted is the exact approved `weathr` command in the already managed pane. Do not replace it with `cargo run`, a shell wrapper, a pipeline, a different binary, or extra recovery services. Do not create a new tmux session or repair other panes as part of this watchdog.

The watchdog must not inject `q`, F1, Ctrl+C, or arbitrary keystrokes as a substitute for state detection. Input belongs to the operator unless the deployment has separately approved a documented, target-locked action.

## Safe manual procedure

1. **Freeze automatic action.** Pause the watchdog or place it in its documented maintenance mode so it cannot race the operator.
2. **Resolve the target read-only.** Confirm the configured session, window, and pane with local tmux inspection. For example, use placeholders with commands such as:
   ```sh
   tmux list-sessions
   tmux list-windows -t <managed-session>
   tmux list-panes -t <managed-session>:<managed-window>
   tmux capture-pane -p -t <managed-session>:<managed-window>.<managed-pane>
   ```
   Never substitute real deployment names, addresses, or identifiers into this public document.
3. **Check identity and state.** Confirm that the selected pane is the managed pane, that its foreground command is exactly `weathr`, and that q/quit, prompt, and alternate-screen observations agree. If they do not, stop and investigate manually; do not broaden the match.
4. **Exit cleanly when possible.** If the verified TUI is responsive, press `q` once and wait for normal terminal cleanup. Ctrl+C is the alternate interactive exit. Do not send keys to an unverified pane.
5. **Start only the approved process.** Once the pane is back at its expected launch point and the target has been rechecked, start exactly `weathr` in that pane. Do not restart any neighboring process or service.
6. **Verify the result.** Confirm that the exact process owns the pane, the alternate screen is active, the HUD/application output is visible, and no unexpected prompt or second process appeared. Resume the 15-second watchdog only after this check.
7. **Escalate ambiguity.** If clean exit or exact identity cannot be confirmed, leave the target stopped and preserve local evidence for the operator. A deliberate stop is safer than a broad kill or an unverified restart.

## Operational hygiene

Keep temporary captures and logs on the local system with appropriate permissions. Before committing documentation, review diffs for IP addresses, PIDs, secrets, host-specific paths, and internal identifiers. This runbook records the safety contract only; it is not a place to publish deployment credentials or server topology.
