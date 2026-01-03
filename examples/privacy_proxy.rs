//! Privacy Proxy Example
//!
//! This example demonstrates:
//! 1. Pattern-based redaction
//! 2. Token vault storage
//! 3. Response reinflation
//!
//! Run with: cargo run --example privacy_proxy

use std::collections::HashMap;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("SuperInstance AI - Privacy Proxy Example\n");

    // Step 1: Initialize privacy proxy
    println!("1. Initializing privacy proxy...");
    let mut proxy = PrivacyProxy::new();
    println!("   ✓ Proxy initialized");
    println!("   Patterns: emails, phones, paths, api_keys, ssn, ip");

    // Step 2: Sample text with sensitive data
    let original = r#"
Please help me debug this issue. My email is john.doe@company.com and
my phone is 555-123-4567. The error log is at /home/john/projects/secret/app.log.
My API key is api_key=sk-abc123def456ghi789jkl012mno345pqr678stu.
Contact my colleague at jane.smith@startup.io or call 555-987-6543.
My SSN is 123-45-6789 for the tax form.
Server IP: 192.168.1.100
    "#;

    println!("\n2. Original text:");
    println!("   {}", original.trim().replace('\n', "\n   "));

    // Step 3: Perform redaction
    println!("\n3. Performing redaction...");
    let (redacted, tokens) = proxy.redact(original);
    
    println!("   Tokens generated:");
    for (token, original_value) in &tokens {
        let preview = if original_value.len() > 30 {
            format!("{}...", &original_value[..30])
        } else {
            original_value.clone()
        };
        println!("     {} → {}", token, preview);
    }

    println!("\n   Redacted text:");
    println!("   {}", redacted.trim().replace('\n', "\n   "));

    // Step 4: Store tokens in vault
    println!("\n4. Storing tokens in vault...");
    let mut vault = TokenVault::new();
    vault.store_batch(&tokens);
    println!("   ✓ {} tokens stored", tokens.len());

    // Step 5: Simulate cloud processing
    println!("\n5. Simulating cloud processing...");
    let cloud_response = format!(
        "I see you're having an issue. Based on the error at [TOKEN_0003], \
         I recommend checking the configuration. You can reach out to [TOKEN_0001] \
         for additional help, or contact [TOKEN_0004] if that's more convenient."
    );
    println!("   Cloud response (with tokens):");
    println!("   {}", cloud_response);

    // Step 6: Reinflate response
    println!("\n6. Reinflating response...");
    let reinflated = vault.reinflate(&cloud_response);
    println!("   Final response:");
    println!("   {}", reinflated);

    // Step 7: Show vault statistics
    println!("\n7. Vault statistics:");
    println!("   Total tokens: {}", vault.len());
    println!("   Types: {:?}", vault.token_types());

    // Step 8: Custom pattern example
    println!("\n8. Custom pattern example...");
    
    proxy.add_pattern(
        "EMPLOYEE_ID",
        r"EMP-\d{6}",
    );
    
    let custom_text = "Employee EMP-123456 reported the issue.";
    let (redacted_custom, _) = proxy.redact(custom_text);
    
    println!("   Original: {}", custom_text);
    println!("   Redacted: {}", redacted_custom);

    println!("\n✓ Example complete!");
    
    Ok(())
}

// Mock implementation for the example

struct PrivacyProxy {
    patterns: Vec<(String, regex::Regex)>,
    token_counter: usize,
}

impl PrivacyProxy {
    fn new() -> Self {
        let patterns = vec![
            ("EMAIL", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"),
            ("PHONE", r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b"),
            ("PATH", r"(?:/[a-zA-Z0-9._-]+)+(?:\.[a-zA-Z0-9]+)?"),
            ("API_KEY", r"(?i)(?:api[_-]?key|secret|token)['\"]?\s*[:=]\s*['\"]?([a-zA-Z0-9_-]{20,})"),
            ("SSN", r"\b\d{3}-\d{2}-\d{4}\b"),
            ("IP", r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b"),
        ];

        let compiled: Vec<_> = patterns
            .into_iter()
            .map(|(name, pat)| (name.to_string(), regex::Regex::new(pat).unwrap()))
            .collect();

        Self {
            patterns: compiled,
            token_counter: 0,
        }
    }

    fn add_pattern(&mut self, name: &str, pattern: &str) {
        let regex = regex::Regex::new(pattern).unwrap();
        self.patterns.push((name.to_string(), regex));
    }

    fn redact(&mut self, text: &str) -> (String, Vec<(String, String)>) {
        let mut result = text.to_string();
        let mut tokens = Vec::new();

        for (name, regex) in &self.patterns {
            let matches: Vec<_> = regex.find_iter(&result).map(|m| m.as_str().to_string()).collect();
            
            for matched in matches {
                let token = format!("[TOKEN_{:04}]", self.token_counter);
                self.token_counter += 1;
                
                tokens.push((token.clone(), matched.clone()));
                result = result.replace(&matched, &token);
            }
        }

        (result, tokens)
    }
}

struct TokenVault {
    tokens: HashMap<String, String>,
    types: HashMap<String, usize>,
}

impl TokenVault {
    fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            types: HashMap::new(),
        }
    }

    fn store_batch(&mut self, tokens: &[(String, String)]) {
        for (token, value) in tokens {
            self.tokens.insert(token.clone(), value.clone());
            
            // Track types (simplified)
            let type_name = if value.contains('@') {
                "email"
            } else if value.chars().filter(|c| c.is_numeric()).count() == 10 {
                "phone"
            } else if value.starts_with('/') {
                "path"
            } else {
                "other"
            };
            
            *self.types.entry(type_name.to_string()).or_insert(0) += 1;
        }
    }

    fn reinflate(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        for (token, value) in &self.tokens {
            result = result.replace(token, value);
        }
        
        result
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }

    fn token_types(&self) -> Vec<String> {
        self.types.keys().cloned().collect()
    }
}
