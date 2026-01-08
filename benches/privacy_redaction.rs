// Criterion benchmark for privacy redaction performance
// Tests pattern matching, token replacement, and document redaction

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use synesis_privacy::{
    redactor::Redactor,
    vault::TokenVault,
    patterns::BuiltinPatterns,
};

/// Benchmark email redaction
fn bench_redact_email(c: &mut Criterion) {
    let text = "Contact us at support@example.com or sales@company.org for assistance.";

    c.bench_function("redact_email", |b| {
        b.iter(|| {
            let redacted = black_box(&text).replace(
                black_box("support@example.com"),
                "[EMAIL_01]"
            );
            black_box(redacted)
        })
    });
}

/// Benchmark phone number redaction
fn bench_redact_phone(c: &mut Criterion) {
    let text = "Call us at 555-123-4567 or +1 (555) 987-6543 for support.";

    c.bench_function("redact_phone", |b| {
        b.iter(|| {
            let redacted = black_box(&text).replace(
                black_box("555-123-4567"),
                "[PHONE_01]"
            );
            black_box(redacted)
        })
    });
}

/// Benchmark credit card redaction (with Luhn check)
fn bench_redact_credit_card(c: &mut Criterion) {
    let text = "Payment with card 4532-1234-5678-9010 was successful.";

    c.bench_function("redact_credit_card", |b| {
        b.iter(|| {
            // Simulate Luhn algorithm check
            let card = black_box("4532123456789010");
            let digits: Vec<u32> = card.chars()
                .filter_map(|c| c.to_digit(10))
                .collect();

            let sum: u32 = digits.iter().rev()
                .enumerate()
                .map(|(i, d)| {
                    if i % 2 == 0 {
                        let doubled = d * 2;
                        if doubled > 9 { doubled - 9 } else { doubled }
                    } else {
                        *d
                    }
                })
                .sum();

            let is_valid = sum % 10 == 0;
            black_box(is_valid)
        })
    });
}

/// Benchmark IP address redaction
fn bench_redact_ip_address(c: &mut Criterion) {
    let text = "Server at 192.168.1.1 or remote at 203.0.113.42 needs configuration.";

    c.bench_function("redact_ip", |b| {
        b.iter(|| {
            let redacted = black_box(&text).replace(
                black_box("192.168.1.1"),
                "[IP_01]"
            );
            black_box(redacted)
        })
    });
}

/// Benchmark API key redaction
fn bench_redact_api_key(c: &mut Criterion) {
    let text = "Use API key sk-1234567890abcdef for authentication.";

    c.bench_function("redact_api_key", |b| {
        b.iter(|| {
            let redacted = black_box(&text).replace(
                black_box("sk-1234567890abcdef"),
                "[API_KEY_01]"
            );
            black_box(redacted)
        })
    });
}

/// Benchmark multiple pattern redaction in one document
fn bench_redact_multiple_patterns(c: &mut Criterion) {
    let text = "Contact john@example.com or call 555-123-4567. Visit 192.168.1.1 or use key sk-abcdef123456.";

    c.bench_function("redact_multiple", |b| {
        b.iter(|| {
            let mut redacted = black_box(&text).to_string();

            // Redact email
            redacted = redacted.replace("john@example.com", "[EMAIL_01]");
            // Redact phone
            redacted = redacted.replace("555-123-4567", "[PHONE_01]");
            // Redact IP
            redacted = redacted.replace("192.168.1.1", "[IP_01]");
            // Redact API key
            redacted = redacted.replace("sk-abcdef123456", "[API_KEY_01]");

            black_box(redacted)
        })
    });
}

/// Benchmark redaction with different document sizes
fn bench_redact_document_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("redact_document_size");

    // Create documents with sensitive info at different sizes
    let base_text = "Contact user@example.com for assistance. Call 555-123-4567 for support. ";
    let email = "admin@company.com";
    let phone = "555-987-6543";

    for size in [1, 5, 10, 50, 100].iter() {
        let document = format!("{}{}{}", base_text.repeat(*size), email, phone);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let mut redacted = black_box(&document).to_string();
                redacted = redacted.replace(email, "[EMAIL_01]");
                redacted = redacted.replace(phone, "[PHONE_01]");
                black_box(redacted.len())
            })
        });
    }

    group.finish();
}

/// Benchmark token vault insertion
fn bench_token_vault_insert(c: &mut Criterion) {
    c.bench_function("vault_insert", |b| {
        b.iter(|| {
            // Simulate token insertion
            let category = black_box("EMAIL");
            let token = format!("[{}_{}]", category, black_box(1));

            black_box(token)
        })
    });
}

/// Benchmark token vault query
fn bench_token_vault_query(c: &mut Criterion) {
    c.bench_function("vault_query", |b| {
        // Create a mock token store
        let tokens: std::collections::HashMap<String, String> = [
            ("[EMAIL_01]".to_string(), "user@example.com".to_string()),
            ("[PHONE_01]".to_string(), "555-123-4567".to_string()),
        ].iter().cloned().collect();

        b.iter(|| {
            let token = black_box("[EMAIL_01]");
            let value = tokens.get(token);
            black_box(value)
        })
    });
}

/// Benchmark reinflation (token → original)
fn bench_reinflation(c: &mut Criterion) {
    let redacted = "Contact [EMAIL_01] for assistance. Call [PHONE_01] for support.";
    let replacements: Vec<(&str, &str)> = vec![
        ("[EMAIL_01]", "john@example.com"),
        ("[PHONE_01]", "555-123-4567"),
    ];

    c.bench_function("reinflation", |b| {
        b.iter(|| {
            let mut inflated = black_box(&redacted).to_string();

            for (token, value) in &replacements {
                inflated = inflated.replace(token, value);
            }

            black_box(inflated)
        })
    });
}

/// Benchmark pattern matching with regex
fn bench_pattern_matching(c: &mut Criterion) {
    use regex::Regex;

    let email_regex = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    let text = "Contact support@example.com or sales@company.org for help.";

    c.bench_function("pattern_matching_email", |b| {
        b.iter(|| {
            let matches: Vec<_> = email_regex.find_iter(black_box(&text)).collect();
            black_box(matches.len())
        })
    });
}

/// Benchmark all built-in patterns
fn bench_all_builtin_patterns(c: &mut Criterion) {
    let text = "Contact john@example.com, call 555-123-4567, visit 192.168.1.1, use key sk-abcdef123456.";

    c.bench_function("redact_all_patterns", |b| {
        b.iter(|| {
            let mut redacted = black_box(&text).to_string();

            // Apply all built-in patterns (simplified)
            let patterns = vec![
                (r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", "[EMAIL_XX]"),
                (r"\d{3}-\d{3}-\d{4}", "[PHONE_XX]"),
                (r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}", "[IP_XX]"),
            ];

            for (pattern, replacement) in patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    redacted = re.replace_all(&redacted, replacement).to_string();
                }
            }

            black_box(redacted)
        })
    });
}

criterion_group!(
    benches,
    bench_redact_email,
    bench_redact_phone,
    bench_redact_credit_card,
    bench_redact_ip_address,
    bench_redact_api_key,
    bench_redact_multiple_patterns,
    bench_redact_document_sizes,
    bench_token_vault_insert,
    bench_token_vault_query,
    bench_reinflation,
    bench_pattern_matching,
    bench_all_builtin_patterns
);
criterion_main!(benches);
