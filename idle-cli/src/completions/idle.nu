# Nushell completions for `idlescreen` — kept in sync with cli.rs.

def "nu-complete idlescreen subcommands" [] {
    [
        { value: "status", description: "Show daemon state (-j/--json)" }
        { value: "config", description: "View or change daemon configuration" }
        { value: "enable", description: "Turn idle screensaver on" }
        { value: "disable", description: "Turn idle screensaver off" }
        { value: "timeout", description: "Set or show idle timeout (1-240 min)" }
        { value: "saver", description: "Show or change the active saver" }
        { value: "list", description: "List installed savers (-j/--json)" }
        { value: "inhibitors", description: "List active idle inhibitors" }
        { value: "inhibit", description: "Run a command while inhibiting idle" }
        { value: "preview", description: "Preview a saver fullscreen" }
        { value: "stop", description: "Stop preview or idle presentation" }
        { value: "fps-overlay", description: "FPS overlay on|off|status" }
        { value: "render-scale", description: "Render scale 0.25-1.0|default" }
        { value: "interactive", description: "Interactive console panel" }
        { value: "restart", description: "Restart the idle-daemon service" }
        { value: "logs", description: "Show daemon logs (-f/--follow, -n)" }
        { value: "doctor", description: "Run diagnostics (-f/--fix, -j/--json)" }
        { value: "clean", description: "Remove stale run state (-n dry-run)" }
        { value: "completion", description: "Print a shell completion script" }
        { value: "bug-report", description: "Sanitized diagnostics bundle" }
        { value: "self-update", description: "Upgrade packages (-c check only)" }
        { value: "tui", description: "Launch the full-screen TUI" }
        { value: "version", description: "Print CLI version" }
        { value: "about", description: "Version plus project info" }
        { value: "help", description: "Show help" }
    ]
}

def "nu-complete idlescreen shells" [] {
    [bash zsh fish nushell powershell elvish]
}

def "nu-complete idlescreen savers" [] {
    ^idlescreen list --json | from json
}

export extern "idlescreen" [
    command?: string@"nu-complete idlescreen subcommands"
    args?: string
    --json(-j)        # Machine-readable output
    --fix(-f)         # doctor: attempt repairs
    --timeout(-t): int # preview: auto-stop seconds
    --quiet(-q)       # suppress confirmations
    --help(-h)
    --version(-V)
]

export extern "idlescreen completion" [
    shell?: string@"nu-complete idlescreen shells"
]

export extern "idlescreen preview" [
    name?: string@"nu-complete idlescreen savers"
    --timeout(-t): int
    --help(-h)
]

export extern "idlescreen saver set" [
    name?: string@"nu-complete idlescreen savers"
]

export extern "idlescreen logs" [
    --follow(-f)
    --lines(-n): int
    --help(-h)
]
