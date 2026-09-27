#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use anyhow::{anyhow, Result};
use debug_print::{debug_println as dprintln};
//use std::str::FromStr;
use std::{time::Duration};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::{Write, BufRead, BufReader};
use std::path::PathBuf;

use crate::utils::helpers::is_win7_or_greater;
use super::GLOBAL_SETTINGS;

use base64::{prelude::BASE64_STANDARD, Engine};

struct NetscapeCookie {
    domain: String,
    include_subdomains: String, 
    path: String, 
    secure: String, 
    exp: String,
    name: String,
    value: String
}
pub struct ClientBuilder {
	emulation: Option<String>,
	default_headers: Option<HashMap<String, String>>,
	timeout: Option<Duration>,
	//user_agent: Option<String>,
	gzip: bool,
	use_proxy: bool,
	expect_binary: bool,
	expect_raw: bool,
	netscape_cookies_write: Option<PathBuf>,
	netscape_cookies_send: Option<PathBuf>
}
impl ClientBuilder {

	pub fn emulation(mut self, emulation: impl Into<String>) -> Self {
		self.emulation = Some(emulation.into());
        self
	}
	pub fn default_headers(mut self, default_headers: HashMap<String, String>) -> Self {
		self.default_headers = Some(default_headers);
        self
	}
	pub fn timeout(mut self, timeout: Duration) -> Self {
		self.timeout = Some(timeout);
        self
	}
	/*#[allow(dead_code)]
	pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
		self.user_agent = Some(user_agent.into());
        self
	}*/
	pub fn gzip(mut self, gzip: bool) -> Self {
		self.gzip = gzip;
        self
	}
	pub fn proxy(mut self, use_proxy: bool) -> Self {
		self.use_proxy = use_proxy;
        self
	}
	#[allow(dead_code)]
	pub fn expect_binary(mut self, b: bool) -> Self {
		self.expect_binary = b;
        self
	}
	#[allow(dead_code)]
	pub fn expect_raw(mut self, b: bool) -> Self {
		self.expect_raw = b;
        self
	}
	pub fn netscape_cookies_write(mut self, path: PathBuf) -> Self {
		self.netscape_cookies_write = Some(path);
        self
	}
	pub fn netscape_cookies_send(mut self, path: PathBuf) -> Self {
		self.netscape_cookies_send = Some(path);
        self
	}
	pub fn build(self) -> Result<Client> {
        Ok(Client {
        	//lib: self.lib,
        	emulation: self.emulation,
        	default_headers: self.default_headers,
        	timeout: self.timeout,
        	//user_agent: self.user_agent,
        	gzip: self.gzip,
        	use_proxy: self.use_proxy,
        	expect_binary: self.expect_binary,
        	expect_raw: self.expect_raw,
        	netscape_cookies_write: self.netscape_cookies_write,
        	netscape_cookies_send: self.netscape_cookies_send,

        	post: None,
        	body: None,
        	get: None,
        	query: None,
        	version: None,
        })
    }
}


#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Client {
	//lib: ReqLib,
	emulation: Option<String>,
	default_headers: Option<HashMap<String, String>>,
	timeout: Option<Duration>,
	//user_agent: Option<String>,
	gzip: bool,
	use_proxy: bool,
	expect_binary: bool,
	expect_raw: bool,
	netscape_cookies_write: Option<PathBuf>,
	netscape_cookies_send: Option<PathBuf>,

	post: Option<String>,
	body: Option<String>,
	get: Option<String>,
	query: Option<Vec<(String, String)>>,
	version: Option<String>,
}
impl Client {
	pub fn builder() -> ClientBuilder {
	    ClientBuilder {
	        emulation: None,
			default_headers: None,
			timeout: None,
			//user_agent: None,
			gzip: false,
			use_proxy: false,
			expect_binary: false,
			expect_raw: false,
			netscape_cookies_write: None,
			netscape_cookies_send: None
		}
	}
	pub fn post(mut self, url: impl Into<String>) -> Self {
		self.post = Some(url.into());
		self.get = None;
        self
	}
	pub fn body(mut self, body: impl Into<String>) -> Self {
		self.body = Some(body.into());
        self
	}
	pub fn get(mut self, url: impl Into<String>) -> Self {
		self.get = Some(url.into());
		self.post = None;
        self
	}
	pub fn query<I, K, V>(mut self, query: I) -> Self 
	where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
    	let vec: Vec<(String, String)> = query
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self.query = Some(vec);
        self
	}
	pub fn version(mut self, v: impl Into<String>) -> Self {
		self.version = Some(v.into());
        self
	}
	pub fn expect_binary(mut self, f: bool) -> Self {
		self.expect_binary = f;
        self
	}
	pub fn expect_raw(mut self, f: bool) -> Self {
		self.expect_raw = f;
        self
	}
	pub fn send(mut self) -> Result<Response<ureq::Body>> {
		if GLOBAL_SETTINGS.use_proxy_global {
			self.use_proxy = true;
		}
		if self.emulation.is_some() && !self.expect_raw {
			run_wreq_cli(self)
		} else {
			make_request_with_ureq(self)
		}
	}
}



pub struct Response<T> {
	status: StatusCode,
	pub headers: Option<HashMap<String, String>>,
	text: Result<String>,
	bytes: Result<Vec<u8>>,
	raw: Result<http::response::Response<T>>,
}
impl<T> Response<T> {
	pub fn status(&self) -> StatusCode {
		self.status.clone()
	}
	pub fn text(&self) -> Result<String> {
		self.text.as_ref().map(|v| v.clone()).map_err(|e| anyhow::anyhow!("{e}"))
	}
	pub fn bytes(&self) -> Result<Vec<u8>> {
		self.bytes.as_ref().map(|v| v.clone()).map_err(|e| anyhow::anyhow!("{e}"))
	}
	pub fn raw(self) -> Result<http::response::Response<T>> {
		self.raw.map_err(|e| anyhow::anyhow!("{e}"))
	}
}


#[derive(Debug, Clone)]
pub struct StatusCode {
	is_success: bool,
	to_u16: u16,
	description: String
}

impl StatusCode {
	pub fn is_success(&self) -> bool {
		self.is_success
	}
	pub fn to_u16(&self) -> u16 {
		self.to_u16
	}
}
impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.description)
    }
}

fn configure_request<B>(req: ureq::RequestBuilder<B>, request: Client, uri: &str) -> (ureq::RequestBuilder<B>, Vec<NetscapeCookie>) {
	let mut req = req;
	    match request.version.as_deref() {
	        Some("HTTP_11") => {
	            req = req.version(ureq::http::Version::HTTP_11);
	        }
	        Some("HTTP_2") => {
	            req = req.version(ureq::http::Version::HTTP_2);
	        }
	        Some(&_) => {}
	        None => {}
	    };

		if let Some(ref default_headers) = request.default_headers {
	        for (key, value) in default_headers {
	            req = req.header(key, value);
	        }
	    }

	    if let Some(query) = request.query {
	    	req = req.query_pairs(query);
	    }

	    let mut old_cookies: Vec<NetscapeCookie> = Vec::new();

	    if let Some(c) = request.netscape_cookies_send {
	    	let arr = read_cookies(c, uri)
	    		.unwrap_or((Vec::new(), Vec::new()));
	    	old_cookies = arr.0;
	    	let cookies_val = arr.1.join("; ");
            dprintln!("{}", &cookies_val);
	    	req = req.header("Cookie", cookies_val);
	    }

	    (req, old_cookies)
	}

fn make_request_with_ureq(request: Client) -> Result<Response<ureq::Body>> {

	/*
	emulation: Option<Emulation>,
	default_headers: Option<HashMap<String, String>>,
	timeout: Option<Duration>,
	user_agent: Option<String>,
	gzip: bool,
	use_proxy: bool,
	expect_binary: bool,

	post: Option<String>,
	body: Option<String>,
	get: Option<String>,
	query: Option<Vec<(String, String)>>,*/

	let mut proxy = None;
	if request.use_proxy && let Some(proxy_settings) = &GLOBAL_SETTINGS.proxy {
		let parts: Vec<&str> = proxy_settings.url.split("://").collect();
	    let protocol = parts[0];
	    let port = parts[1];
	    let mut formatted_proxy = format!("{}://{}", protocol, port);
       
        if let Some(username) = &proxy_settings.username && let Some(password) = &proxy_settings.password {
        	formatted_proxy = format!("{}://{}:{}@{}", protocol, username, password, port);
    	}

    	let ureq_proxy = ureq::Proxy::new(&formatted_proxy)?;
    	proxy = Some(ureq_proxy);
    }

    let config = ureq::Agent::config_builder()
        .timeout_global(request.timeout)
        .proxy(proxy)
        .build();

    let agent: ureq::Agent = config.into();
	let mut response;

	let host;
	let old_cookies: Vec<NetscapeCookie>;
    if let Some(ref url) = request.post {
    	let client = agent.post(url);
    	host = client.uri_ref().unwrap().host().map(|host| host.to_string());
    	let (client, cookies) = configure_request(client, request.clone(), url);
    	old_cookies = cookies;
    	if let Some(b) = request.body {
	    	response = client.send(b)?;
	    } else {
	    	response = client.send_empty()?;
	    }
    } else if let Some(ref url) = request.get {
    	let client = agent.get(url);
    	host = client.uri_ref().unwrap().host().map(|host| host.to_string());
    	let (client, cookies) = configure_request(client, request.clone(), url);
    	old_cookies = cookies;
    	response = client.call()?;
    } else {
    	return Err(anyhow!("url requied"));
    }

    let status = response.status();
    let status_descr = format!("{}", status);
    //let body = response.body_mut();
    let resp_headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string().to_lowercase(),
                value.to_str().unwrap_or("").to_string(),
            )
        })
        .collect();
    
    if let Some(c) = request.netscape_cookies_write && let Some(ref host) = host {
    	let mut netscape_cookies: Vec<NetscapeCookie> = Vec::new();
    	let cookies_raw = response.headers().get_all(http::header::SET_COOKIE);
    	for val in cookies_raw.iter() {
    		let cookie_str = match val.to_str() {
	            Ok(s) => s,
	            Err(_) => continue,
        	};
    		if let Ok(cookie) = cookie::Cookie::parse(cookie_str) {
    			let domain = cookie.domain_raw();
                let include_subdomains = if domain.is_some() { "TRUE" } else { "FALSE" };
                let domain = domain.unwrap_or(host);

                let path = cookie.path().unwrap_or("/");
                let secure = if cookie.secure().unwrap_or(false) { "TRUE" } else { "FALSE" };

                let max_age = cookie.max_age()
                    .map(|dt| {
                        timestamp_from_max_age(dt.try_into().unwrap_or(std::time::Duration::from_secs(0)))
                    }).unwrap_or(0);
                  
                let expires = cookie.expires_datetime()
                    .map(|dt| {
                        timestamp_from_expires_str(dt.into())
                    }).unwrap_or(0);

                let exp = if max_age > 1 {max_age} else {expires};
                let name = cookie.name();
                let value = cookie.value();
                //let prefix = if cookie.http_only().unwrap_or(false) { "#HttpOnly_" } else { "" };
                netscape_cookies.push(NetscapeCookie{
                    domain: domain.to_string(), 
                    include_subdomains: include_subdomains.into(), 
                    path: path.into(), 
                    secure: secure.into(), 
                    exp: exp.to_string(), 
                    name: name.into(), 
                    value: value.into()
                });
    		}
        }
        let _ = write_cookies(c, netscape_cookies, old_cookies);
    }

    let mut response_raw: Result<http::response::Response<ureq::Body>> = Err(anyhow!("e"));
    let mut response_text: Result<String> = Err(anyhow!("e"));
    let mut response_bytes: Result<Vec<u8>> = Err(anyhow!("e"));

	//TODO
    if request.expect_binary {
    	response_bytes = response.body_mut().read_to_vec().map_err(|e| anyhow!(e));
    } else if request.expect_raw {
    	response_raw = Ok(response);
    } else {
    	response_text = response.body_mut().read_to_string().map_err(|e| anyhow!(e));
    }
    
    Ok(
    	Response {
        	text: response_text,
        	bytes: response_bytes,
        	raw: response_raw,
        	status: StatusCode {
            		is_success: status.is_success(),
            		to_u16: status.as_u16(),
            		description: status_descr
            },
            headers: Some(resp_headers),
    	}
    )
}

fn run_wreq_cli<T>(request: Client) -> Result<Response<T>> {

	/*lib: ReqLib,
	emulation: Option<Emulation>,
	default_headers: Option<HashMap<String, String>>,
	timeout: Option<Duration>,
	user_agent: Option<String>,
	gzip: bool,
	use_proxy: bool,
	expect_binary: bool,

	post: Option<String>,
	body: Option<String>,
	get: Option<String>,
	query: Option<Vec<(String, String)>>,*/

	let arg_url: String;
	let arg_X: String;

	let mut arg_proxy: Option<String> = None;
	let mut arg_U: Option<String> = None;

	let mut args_H: Option<Vec<String>> = None;
	let mut arg_d: Option<String> = None;
	let mut args_data: Option<Vec<String>> = None;

	let mut arg_emulation: Option<String> = None;
	let mut arg_http: Option<String> = None;
	let mut arg_timeout: Option<String> = None;
	let mut arg_c: Option<std::ffi::OsString> = None;
	let mut arg_b: Option<std::ffi::OsString> = None;
	let mut arg_gzip = false;
	let mut arg_base64 = false;


	if let Some(post) = request.post {
    	arg_url = post;
    	arg_X = "-X=POST".to_string();
    	if let Some(data) = request.body.clone() {
    		arg_d = Some(format!("-d {}", data));
    	}
    } else if let Some(get) = request.get {
    	arg_url = get;
    	arg_X = "-X=GET".to_string();
    } else {
    	return Err(anyhow!("url is requed"));
    }

    /*if let Some(data) = request.body.clone() {
    	arg_d = Some(format!("-d \"{}\"", data));
    }*/

	if request.use_proxy && let Some(proxy_settings) = &GLOBAL_SETTINGS.proxy {
        arg_proxy = Some(format!("--proxy={}", proxy_settings.url.clone()));
        if let Some(username) = &proxy_settings.username && let Some(password) = &proxy_settings.password {
        	arg_U = Some(format!("-U {username}:{password}"))
    	}        
    }

    if let Some(default_headers) = request.default_headers {
    	args_H = Some(
    		default_headers
		        .iter()
		        .flat_map(|(key, value)| {
		            let header_string = format!("{key}:{value}");
		            vec!["-H".to_string(), header_string]
		        })
		        .collect()
        )
    }

    if let Some(query) = request.query {
    	args_data = Some(
    		query
		        .iter()
		        .flat_map(|(key, value)| {
		            let data_string = format!("{key}={value}");
		            vec!["--data-urlencode".to_string(), data_string]
		        })
		        .collect()
		)
    }
    
    if let Some(v) = request.version {
    	arg_http = Some(format!("--http-version={}", v));
    }
    if let Some(e) = request.emulation {
    	arg_emulation = Some(format!("--emulation={}", e));
    }
    if let Some(t) = request.timeout {
    	arg_timeout = Some(format!("--connect-timeout={}", t.as_secs()));
    }
    if let Some(c) = request.netscape_cookies_write {
    	let mut p = std::ffi::OsString::from("-c ");
    	p.push(&c);
    	arg_c = Some(p);
    }
    if let Some(b) = request.netscape_cookies_send {
    	let mut p = std::ffi::OsString::from("-b ");
    	p.push(&b);
    	arg_b = Some(p);
    }
    if request.gzip {
    	arg_gzip = true;
    }
    if request.expect_binary {
    	arg_base64 = true;
    }
    

    /*if let Some(data) = request.body {
    	let bytes = bincode::serialize(&data).expect("Bincode error");
	    let encoded = BASE64_STANDARD.encode(bytes);
    }*/

	let working_dir = std::env::current_dir().unwrap();
	use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let command = ".\\wreq_cli".to_string();
        
    if which::which(&command).is_ok() {
    	dprintln!("cmd found");
        let mut child = std::process::Command::new(working_dir.join(&command));

        child.arg(&arg_url);
        child.arg(&arg_X);
        if let Some(arr) = args_H {
        	child.args(&arr);
        }
        if let Some(arr) = args_data {
        	child.args(&arr);
        }
        if let Some(d) = arg_d {
        	child.arg(&d);
        }
        if let Some(p) = arg_proxy {
        	child.arg(&p);
        	if let Some(u) = arg_U {
        		child.arg(&u);
        	}
        }
        if let Some(e) = arg_emulation {
        	child.arg(&e);
        }
        if let Some(h) = arg_http {
        	child.arg(&h);
        }
        if let Some(t) = arg_timeout {
        	child.arg(&t);
        }
        if let Some(a) = arg_c {
        	child.arg(&a);
        	dprintln!("{:?}", &a);
        }
        if let Some(a) = arg_b {
        	child.arg(&a);
        	dprintln!("{:?}", &a);
        }
        if arg_gzip {
        	child.arg("--gzip");
        }
        if arg_base64 {
        	child.arg("--base64-response");
        }
        
        child
            .creation_flags(CREATE_NO_WINDOW)
            .current_dir(working_dir);

        let mut child_err = "".to_string();
        let mut child_output = "".to_string();
        if !is_win7_or_greater() {
            let output_file = File::create("wreq_output.tmp")?;
            let output_err_file = File::create("wreq_output_err.tmp")?;

            {
                let child = child
                	.stdin(std::process::Stdio::null()) 
                	.stdout(std::process::Stdio::from(output_file)) 
                	.stderr(std::process::Stdio::from(output_err_file));
                let mut child = child.spawn()?;
                let status = child.wait()?;
            }

            if let Ok(mut file) = File::open("wreq_output.tmp") {
                file.read_to_string(&mut child_output)?;
            }
            if let Ok(mut file) = File::open("wreq_output_err.tmp") {
                file.read_to_string(&mut child_err)?;
            }
            if let Err(e) = std::fs::remove_file("wreq_output.tmp") {
                println!("error remove file: {}", e);
            }
            if let Err(e) = std::fs::remove_file("wreq_output_err.tmp") {
                println!("error remove file: {}", e);
            }
        } else {
            let child = child.output()?;
        	child_err = String::from_utf8_lossy(&child.stderr).into_owned();
        	child_output = String::from_utf8_lossy(&child.stdout).into_owned();
        }

        
        dprintln!("{}", child_err);
        dprintln!("{}", child_output);

    	let error = extract_text("<WREQ_ERROR_BEGIN>", "<WREQ_ERROR_END>", &child_output);
    	if !error.is_empty() {
    		return Err(anyhow!(error));
    	}
    	
    	let status_success = extract_text("<WREQ_IS_SUCCESS_BEGIN>", "<WREQ_IS_SUCCESS_END>", &child_output);
    	let status_success = status_success == "1";
    	let status_u16 = extract_text("<WREQ_U16_STATUS_BEGIN>", "<WREQ_U16_STATUS_END>", &child_output);
    	let status_u16 = status_u16.parse::<u16>();
    	let status_u16 = status_u16.unwrap_or(200_u16);
    	let response_text = extract_text("<WREQ_PAYLOAD_BEGIN>", "<WREQ_PAYLOAD_END>", &child_output);
    	let status_descr = extract_text("<WREQ_STATUS_BEGIN>", "<WREQ_STATUS_END>", &child_output);

    	let mut response_bytes: Result<Vec<u8>> = Err(anyhow!("bytes support only in base64"));
    	if arg_base64 {
    		response_bytes = BASE64_STANDARD.decode(&response_text).map_err(|e| anyhow!(e));
    	}
    	
    	Ok(
        	Response {
            	text: Ok(response_text),
            	bytes: response_bytes,
            	raw: Err(anyhow!("wreq_cli doesn't support returning raw responses")),
            	status: StatusCode {
	            		is_success: status_success,
	            		to_u16: status_u16,
	            		description: status_descr
	            },
	            headers: None, //TODO
        	}
        )
    } else {
        return Err(anyhow!("wreq_cli not found"));
    }
}



fn extract_text(start_tag: &str, end_tag: &str, input: &str) -> String {
    //TODO: utils
    let start_idx = input.find(start_tag);
    let end_idx = input.rfind(end_tag).unwrap_or(input.len());
    let text_start = if let Some(start_idx) = start_idx { 
        start_idx + start_tag.len() 
    } else {
        return "".to_string();
    };

    if text_start <= end_idx {
        input[text_start..end_idx].to_string()
    } else {
    	"".to_string()
    }
}

fn read_cookies(f: PathBuf, url: &str) -> Result<(Vec<NetscapeCookie>, Vec<String>)> {
    let mut netscape_cookies: Vec<NetscapeCookie> = Vec::new();
    if !f.exists() {
        return Err(anyhow!("path"));
    }
    let parsed_url = url.parse::<http::Uri>().unwrap();
    let request_path = parsed_url.path();
    let request_domain = parsed_url.host().unwrap();

    let mut cookie_pairs = Vec::new();
    let file = File::open(f)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line?;
        //println!("{:?}", &line);
        if (line.starts_with('#') && !line.starts_with("#HttpOnly")) || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 7 {
            let name = parts[5];
            let value = parts[6];
            let cookie_path = parts[2];
            let cookie_domain = parts[0].to_string().replace("#HttpOnly_", "");
            let include_subdomains = parts[1].to_string();

            netscape_cookies.push(
                NetscapeCookie {domain: cookie_domain.clone(), include_subdomains: include_subdomains.clone(), path: cookie_path.to_string(), secure: parts[3].to_string(), exp: parts[4].to_string(), name: name.to_string(), value: value.to_string()}
            );

            if !domain_match(request_domain, &cookie_domain, &include_subdomains) {
                continue;
            }
            if !path_match(request_path, cookie_path) {
                continue;
            }
            if let Ok(expires) = parts[4].parse::<i64>() {
                let now = get_timestamp();
                if expires > 1 && expires < now {
                    continue;
                }
            }

            cookie_pairs.push(format!("{}={}", name, value));
        }
    }
    Ok((netscape_cookies, cookie_pairs))
}

fn write_cookies(f: PathBuf, new_cookies: Vec<NetscapeCookie>, old_cookies: Vec<NetscapeCookie>) -> Result<()> {
    let mut file = File::create(f)?;
    writeln!(file, "# Netscape HTTP Cookie File\n")?;
    let cookies = merge_cookies(new_cookies, old_cookies);
    for cookie in cookies {
        let NetscapeCookie {domain, include_subdomains, path, secure, exp, name, value} = cookie;
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            domain, include_subdomains, path, secure, exp, name, value
        )?;
    }
    Ok(())
}

fn merge_cookies(
    new_cookies: Vec<NetscapeCookie>,
    old_cookies: Vec<NetscapeCookie>
) -> Vec<NetscapeCookie> {
    let mut merged: std::collections::HashMap<(String, String, String), NetscapeCookie> = std::collections::HashMap::new();

    for cookie in old_cookies {
        let key = (cookie.domain.clone(), cookie.path.clone(), cookie.name.clone());
        merged.insert(key, cookie);
    }
    
    for cookie in new_cookies {
        let key = (cookie.domain.clone(), cookie.path.clone(), cookie.name.clone());
        merged.insert(key, cookie);
    }

    merged.into_values().collect()
}

fn timestamp_from_max_age(max_age_secs: std::time::Duration) -> i64 {
    let expire_time = std::time::SystemTime::now() + max_age_secs;
    expire_time
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn timestamp_from_expires_str(expires: std::time::SystemTime) -> i64 {
    expires
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn get_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn domain_match(request_host: &str, cookie_host: &str, include_subdomains: &str) -> bool {
    //println!("request_host: {}, cookie_host: {}", request_host, cookie_host);
    if (include_subdomains == "TRUE" && request_host.ends_with(cookie_host)) 
	|| request_host == cookie_host {
        return true;
    }
    false
}

fn path_match(request_path: &str, cookie_path: &str) -> bool {
    //println!("request_path: {}, cookie_path: {}", request_path, cookie_path);
    if request_path == cookie_path {
        return true;
    }
    if request_path.starts_with(cookie_path) {
        if cookie_path.ends_with('/') {
            return true;
        }

        //cookie_path = "/api", request_path = "/api/v1"
        if request_path.as_bytes().get(cookie_path.len()) == Some(&b'/') {
            return true;
        }
    }
    false
}