



if (Get-Command deno -ea SilentlyContinue) {
    Write-Host "deno command exists"
} else {
    $deno_dwlnd = Read-Host -Prompt "Download standalone deno runtime?(Y/N)"
    if ($deno_dwlnd -eq 'y' -or $deno_dwlnd -eq 'Y') {
        $deno_version = curl.exe --ssl-revoke-best-effort -s "https://dl.deno.land/release-latest.txt"
        #$deno_version = Invoke-WebRequest -Uri "https://www.example.com" | Select-Object -ExpandProperty Content
        $DownloadUrl = "https://dl.deno.land/release/${deno_version}/deno-x86_64-pc-windows-msvc.zip"
        $DenoZip = "deno.zip"
        $DenoExe = "deno.exe"

        Write-Host $DownloadUrl
        curl.exe --ssl-revoke-best-effort -Lo $DenoZip $DownloadUrl
        #[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        #Invoke-WebRequest -Uri $DownloadUrl -OutFile $DenoZip
        tar.exe xf $DenoZip
        Remove-Item $DenoZip

        Write-Output "Deno was installed successfully to ${DenoExe}"
    }
    
}