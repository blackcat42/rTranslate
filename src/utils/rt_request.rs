#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use anyhow::{anyhow, Result};
use debug_print::{debug_println as dprintln};
//use std::str::FromStr;
use std::{time::Duration};
use std::collections::HashMap;

use super::GLOBAL_SETTINGS;

use base64::{prelude::BASE64_STANDARD, Engine};

pub struct ClientBuilder {
	emulation: Option<String>,
	default_headers: Option<HashMap<String, String>>,
	timeout: Option<Duration>,
	//user_agent: Option<String>,
	gzip: bool,
	use_proxy: bool,
	expect_binary: bool,
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
	pub fn send(self) -> Result<Response> {
		if self.emulation.is_some() {
			run_wreq_cli(self)
		} else {
			make_request_with_ureq(self)
		}
	}
}



pub struct Response {
	status: StatusCode,
	text: Result<String>,
	bytes: Result<Vec<u8>>,
}
impl Response {
	pub fn status(&self) -> StatusCode {
		self.status.clone()
	}
	pub fn text(&self) -> Result<String> {
		self.text.as_ref().map(|v| v.clone()).map_err(|e| anyhow::anyhow!("{e}"))
	}
	pub fn bytes(&self) -> Result<Vec<u8>> {
		self.bytes.as_ref().map(|v| v.clone()).map_err(|e| anyhow::anyhow!("{e}"))
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

fn configure_request<B>(req: ureq::RequestBuilder<B>, request: Client) -> ureq::RequestBuilder<B> {
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
	    req
	}

fn make_request_with_ureq(request: Client) -> Result<Response> {

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

    if let Some(ref url) = request.post {
    	let client = agent.post(url);
    	let client = configure_request(client, request.clone());
    	if let Some(b) = request.body {
	    	response = client.send(b)?;
	    } else {
	    	response = client.send_empty()?;
	    }
    } else if let Some(ref url) = request.get {
    	let client = agent.get(url);
    	let client = configure_request(client, request.clone());
    	response = client.call()?;
    } else {
    	return Err(anyhow!("url requied"));
    }

    let status = response.status();
    let status_descr = format!("{}", status);
    let body = response.body_mut();
    let mut response_text: Result<String> = Err(anyhow!("e"));
    let mut response_bytes: Result<Vec<u8>> = Err(anyhow!("e"));

    if request.expect_binary {
    	response_bytes = body.read_to_vec().map_err(|e| anyhow!(e));
    } else {
    	response_text = body.read_to_string().map_err(|e| anyhow!(e));
    }
    
    Ok(
    	Response {
        	text: response_text,
        	bytes: response_bytes,
        	status: StatusCode {
            		is_success: status.is_success(),
            		to_u16: status.as_u16(),
            		description: status_descr
            }
    	}
    )
}

fn run_wreq_cli(request: Client) -> Result<Response> {

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
        let mut child_output = std::process::Command::new(working_dir.join(&command));

        child_output.arg(&arg_url);
        child_output.arg(&arg_X);
        if let Some(arr) = args_H {
        	child_output.args(&arr);
        }
        if let Some(arr) = args_data {
        	child_output.args(&arr);
        }
        if let Some(d) = arg_d {
        	child_output.arg(&d);
        }
        if let Some(p) = arg_proxy {
        	child_output.arg(&p);
        	if let Some(u) = arg_U {
        		child_output.arg(&u);
        	}
        }
        if let Some(e) = arg_emulation {
        	child_output.arg(&e);
        }
        if let Some(h) = arg_http {
        	child_output.arg(&h);
        }
        if let Some(t) = arg_timeout {
        	child_output.arg(&t);
        }
        if arg_gzip {
        	child_output.arg("--gzip");
        }
        if arg_base64 {
        	child_output.arg("--base64-response");
        }
        
        child_output
            .creation_flags(CREATE_NO_WINDOW)
            .current_dir(working_dir);
        let child_output = child_output.output()?;
        let child_err = String::from_utf8_lossy(&child_output.stderr).into_owned();
        let child_output = String::from_utf8_lossy(&child_output.stdout).into_owned();
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
            	status: StatusCode {
	            		is_success: status_success,
	            		to_u16: status_u16,
	            		description: status_descr
	            }
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
