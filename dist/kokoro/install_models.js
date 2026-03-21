import * as fs from 'node:fs';
import * as https from 'node:https';

function run() {
    const file = fs.createWriteStream('models/onnx-community/Kokoro-82M-v1.0-ONNX/onnx/model_fp16.onnx');

    const request = https
        .get('https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/resolve/main/onnx/model_fp16.onnx?download=true', (response) => {
            response.pipe(file);
            file.on('finish', () => {
                file.close(() => {
                    console.log('File downloaded successfully');
                });
            });
        })
        .on('error', (err) => {
            fs.unlink(path, () => {
                // Delete the partially downloaded file
                console.error('Error downloading file:', err.message);
            });
        });
    request.on('error', (err) => {
        console.error('Error making request:', err.message);
    });
}

run();
