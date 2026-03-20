//! Static documentation strings for YatsuScript built-in functions and keywords.
//!
//! These are emitted as hover and completion documentation.

/// Built-in functions: (name, signature, doc)
pub const BUILTINS: &[(&str, &str, &str)] = &[
    (
        "print",
        "print(...args)",
        "Prints all arguments separated by spaces, then a newline.\n\n```ys\nprint(\"hello\", 42, true)\n```",
    ),
    (
        "len",
        "len(x) -> number",
        "Returns the length of a string, list, object, or range.\n\n```ys\nlen(\"abc\")     // 3\nlen([1, 2, 3]) // 3\n```",
    ),
    (
        "str",
        "str(x) -> string",
        "Converts a value to its string representation.\n\n```ys\nstr(42)   // \"42\"\nstr(true) // \"true\"\n```",
    ),
    (
        "time",
        "time() -> number",
        "Returns the current Unix timestamp as a floating-point number of seconds.",
    ),
    (
        "timestamp",
        "timestamp() -> Timestamp",
        "Returns a `Timestamp` object representing the current instant.\n\nProperties:\n- `.elapsed` — seconds since the timestamp was created\n\n```ys\nlet t: timestamp()\nsleep(100)\nprint(t.elapsed)\n```",
    ),
    (
        "sleep",
        "sleep(ms: number)",
        "Asynchronously sleeps for the given number of milliseconds.\n\n```ys\nsleep(500)\n```",
    ),
    (
        "fetch",
        "fetch(url: string)",
        "Performs an HTTP GET request and prints the status and body to stdout.",
    ),
    (
        "serve",
        "serve(port: number, handler: fn)",
        "Starts a simple HTTP server on `port` and dispatches each request to `handler`.\n\nThe handler receives the raw request text and should return a response string.\n\n```ys\nfn handle(req) {\n    return \"Hello!\"\n}\nserve(9000, handle)\n```",
    ),
];

/// Keywords: (name, doc)
pub const KEYWORDS: &[(&str, &str)] = &[
    ("let",      "Declare an **immutable** variable.\n\n```ys\nlet x: 42\n```"),
    ("mut",      "Declare a **mutable** variable.\n\n```ys\nmut counter: 0\ncounter: counter + 1\n```"),
    ("fn",       "Declare a **function**.\n\n```ys\nfn add(a, b) {\n    return a + b\n}\n```"),
    ("return",   "Return a value from the current function.\n\n```ys\nreturn x + 1\n```"),
    ("if",       "Conditional branch.\n\n```ys\nif x > 0 {\n    print(\"positive\")\n}\n```"),
    ("else",     "Alternative branch of an `if` statement.\n\n```ys\nif x > 0 {\n    print(\"pos\")\n} else {\n    print(\"non-pos\")\n}\n```"),
    ("for",      "`for` loop over a range.\n\n```ys\nfor i in 0..10 {\n    print(i)\n}\n```"),
    ("while",    "`while` loop.\n\n```ys\nmut i: 0\nwhile i < 10 {\n    i: i + 1\n}\n```"),
    ("in",       "Used in `for … in` to specify the range/iterable."),
    ("spawn",    "Spawn a concurrent async task.\n\n```ys\nspawn {\n    print(\"running concurrently\")\n}\n```"),
    ("continue", "Skip to the next iteration of a loop.\n\n```ys\nfor i in 0..10 {\n    if i == 5 { continue }\n    print(i)\n}\n```"),
    ("true",     "Boolean literal `true`."),
    ("false",    "Boolean literal `false`."),
];

/// Look up hover documentation for any word (keyword or built-in).
pub fn get_docs(word: &str) -> Option<String> {
    if let Some(&(_, sig, doc)) = BUILTINS.iter().find(|&&(n, _, _)| n == word) {
        return Some(format!("```ys\n{sig}\n```\n\n{doc}"));
    }
    if let Some(&(_, doc)) = KEYWORDS.iter().find(|&&(n, _)| n == word) {
        return Some(doc.to_string());
    }
    None
}

/// Iterator over all documented words and their documentation.
pub const ITER: &[(&str, &str)] = &[
    ("print", "print(...args)"),
    ("len", "len(x)"),
    ("str", "str(x)"),
    ("time", "time()"),
    ("timestamp", "timestamp()"),
    ("sleep", "sleep(ms)"),
    ("fetch", "fetch(url)"),
    ("serve", "serve(port, handler)"),
    ("let", "Declare variable"),
    ("mut", "Declare mutable variable"),
    ("fn", "Declare function"),
    ("return", "Return value"),
    ("if", "Condition"),
    ("else", "Else branch"),
    ("for", "Loop"),
    ("while", "While loop"),
    ("in", "Iterator"),
    ("spawn", "Spawn task"),
    ("continue", "Continue loop"),
    ("true", "Boolean true"),
    ("false", "Boolean false"),
];
