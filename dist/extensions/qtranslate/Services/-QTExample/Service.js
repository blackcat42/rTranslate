// ============================================================================
// MyNewTranslator for QTranslate
// ============================================================================
// Автор: Имя_Разработчика
// Дата: 2026
// ============================================================================

var SERVICE_URL = "https://translate.googleapis.com";
var SERVICE_ID = 650;  // Пользовательские ID должны быть в диапазоне 500-999

function serviceHeader() {
    return new ServiceHeader(
        SERVICE_ID,
        "MyNewTranslator",
        "Описание сервиса" + Const.NL2 + SERVICE_URL,
        Capability.TRANSLATE
    );
}

function serviceHost() { 
    return SERVICE_URL; 
}

function serviceLink(text, from, to) { 
    if (!text) return SERVICE_URL;
    from = isLanguage(from) ? codeFromLanguage(from) : "auto";
    to = isLanguage(to) ? codeFromLanguage(to) : "en";
    return SERVICE_URL + "/web?text=" + encodeGetParam(text) + "&sl=" + from + "&tl=" + to;
}

function serviceTranslateRequest(text, from, to) {
    // limitSource и prepareSource объявлены глобально в Common.js
    text = encodeUriParam(limitSource(prepareSource(text), 5000));
    
    from = (from === "auto" || !isLanguage(from)) ? "auto" : codeFromLanguage(from);
    to = isLanguage(to) ? codeFromLanguage(to) : "en";
    
    // var body = {
    //     q: text,
    //     source: from,
    //     target: to
    // };

    // Используем stringifyJSON из Common.js вместо нативного JSON.stringify
    return new RequestData(
        HttpMethod.GET, 
        '/translate_a/single?client=gtx&sl='+from+'&dt=t&dt=bd&dj=1&tl='+to+'&text='+text, 
        //stringifyJSON(body), 
        "Content-Type: application/json"
    );
}

function serviceTranslateResponse(original, json, from, to) {
    try {
        console.log(json)
        var data = parseJSON(json); // parseJSON встроена в Common.js
        console.log(data)
        if (!data || !data.sentences[0].trans) {
            throw new Error("Неверный формат ответа API");
        }
        
        var result = data.sentences[0].trans;
        if (isArray(result)) {
            result = result.join(Const.NL);
        }
        
        // trimString(str) - безопасная замена str.trim() из Common.js
        result = trimString(result);
        
        // Нормализация переносов строк и удаление лишних пустых строк
        result = removeEmptyLines(result);
        
        return new ResponseData(result, from, to, "");
    } catch (e) {
        return new ResponseData(original + Const.NL2 + "[⚠️ Ошибка MyNewTranslator: " + e.message + "]", from, to, "");
    }
}

SupportedLanguages=[-1,"auto","af","az","sq","ar","hy","eu","be","bg","ca","zh-CN","zh-TW","hr","cs","da","nl","en","et","fi","tl","fr","gl","de","el","ht","iw","hi","hu","is","id","it","ga","ja","ka","ko","lv","lt","mk","ms","mt","no","fa","pl","pt","ro","ru","sr","sk","sl","es","sw","sv","th","tr","uk","ur","vi","cy","yi","eo","hmn","la","lo","kk","uz","si","tg","te","km","mn","kn","ta","mr","bn","tt"];
