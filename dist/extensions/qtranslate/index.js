
/*function base64encode(text) {
	return btoa(unescape(encodeURIComponent(stringifyJSON(text))));
}*/
function base64decode(base64) {
	return decodeURIComponent(escape(atob(base64)));
}

function cleanIDString(str) {
    if (typeof str !== "string") {
        return "";
    }
    return str.replace(/[^a-zA-Z0-9_а-яА-ЯёЁ\-\s]/g, "");
}

function getArgValue(argName) {
  const index = s_args.findIndex(arg => arg === `--${argName}`);
  if (index === -1) return null;
  const nextArg = s_args[index + 1];
  
  if (nextArg && !nextArg.startsWith("-")) {
    return nextArg;
  }
  return true;
}

let s_args;
if (typeof Deno !== "undefined" && false) {
	eval(`
		let f1 = Deno.readTextFileSync("./extensions/qtranslate/Services/Common.js", "utf-8");
		let f2 = Deno.readTextFileSync("./extensions/qtranslate/Services/DeepL/Service.js", "utf-8");

		const globalEval = (0, eval);
	    globalEval(f1);
	    globalEval(f2);

		s_args = Deno.args;

	`);
} else if (typeof std !== "undefined" && typeof std.loadScript === "function") {
	s_args = scriptArgs;
	let srvc_id = cleanIDString(getArgValue("service-id"));
	std.loadScript('./extensions/qtranslate/Services/Common.js');
	std.loadScript('./extensions/qtranslate/Services/' + srvc_id + '/Service.js');
	
	//TODO no std, make temporary file common.js + service.js
	std = null;
	os = null;
} else {
    console.log("TODO nodejs");
}



//"zh-CN","zh-TW"
addOption('PreferredDomain', 'com');
addOption('GoogleDomain', 'com'); //cn
addOption('LanguageCode', 'ru'); //TODO!!! target lang?
//Options.GoogleTkk
//Options.BingToken
//Options.BingCookie 
//Options.BingKey 
//Options.IG 

let sh = serviceHeader();

if (sh.name === 'Yandex' && sh.id === 11) {
	//TODO: <QT_STATE>
	if (typeof YandexModel === "function"){
		YandexModel.prototype.hasChunks = function () {
			return false;
		};
		yandexModel = new YandexModel('');
	}
	//TODO detect lng
}

if (!!(sh.capabilities & Capability.TRANSLATE)) {
	let qtranslate_eval = getArgValue("qtranslate-eval");
	let arg_decoded = base64decode(qtranslate_eval);
	//console.log("QT-args:", arg_decoded);
	let result = eval(arg_decoded);
	if (result.uri !== undefined) {
		result.uri = serviceHost() + result.uri;
	}
	console.log('<SERVICE_RESPONSE_BEGIN>');
	console.log(stringifyJSON(result));
	console.log('<SERVICE_RESPONSE_END>');
}
