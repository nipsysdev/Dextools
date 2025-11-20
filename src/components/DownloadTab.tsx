import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Input, Progress, Typography } from '@nipsysdev/lsd-react';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { useState } from 'react';

interface DownloadResult {
  cid: string;
  size: number;
  duration_ms: number;
  verified: boolean;
  filepath?: string;
}

export default function DownloadTab() {
  const [cid, setCid] = useState('');
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadResult, setDownloadResult] = useState<DownloadResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const validateCid = (cid: string): boolean => {
    return cid.trim().length > 0;
  };

  const handleCidChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const value = event.target.value;
    setCid(value);
    
    if (value && !validateCid(value)) {
      setError('Invalid CID format');
    } else {
      setError(null);
    }
  };

  const handleDownload = async () => {
    if (!cid || !validateCid(cid)) {
      setError('Please enter a valid CID');
      return;
    }

    try {
      // First, let user select save location
      const savePath = await save({
        filters: [{
          name: 'All Files',
          extensions: ['*']
        }],
        defaultPath: `${cid}.bin`
      });

      if (!savePath) {
        return; // User cancelled
      }

      setIsDownloading(true);
      setDownloadProgress(0);
      setError(null);
      setDownloadResult(null);

      // Simulate progress updates since we don't have real-time progress yet
      const progressInterval = setInterval(() => {
        setDownloadProgress(prev => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return 90;
          }
          return prev + 10;
        });
      }, 200);

      const result = await invoke<DownloadResult>('download_file_from_codex', {
        cid: cid,
        savePath: savePath
      });

      clearInterval(progressInterval);
      setDownloadProgress(100);
      setDownloadResult(result);
    } catch (err) {
      console.error('Download failed:', err);
      setError(`Download failed: ${err}`);
    } finally {
      setIsDownloading(false);
    }
  };

  return (
    <Card className="w-full max-w-md" style={{borderTop: 0}}>
      <CardHeader>
        <CardTitle>Download File from Codex</CardTitle>
        <CardDescription>
          Enter the CID of the file you want to download
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Input
          placeholder="Enter CID (e.g., Qm... or bafy...)"
          value={cid}
          onChange={handleCidChange}
          label="Content Identifier (CID)"
          supportingText="Enter the CID of the file you want to download"
          error={!!error}
        />
        
        {isDownloading && (
          <div className="space-y-2">
            <Typography variant="body2">
              Downloading from Codex...
            </Typography>
            <Progress value={downloadProgress} />
          </div>
        )}
        
        {downloadResult && (
          <div className="space-y-2 p-4 bg-lsd-surface-secondary rounded-md">
            <Typography variant="body2" color="primary">
              Download successful!
            </Typography>
            <Typography variant="body2">
              CID: <span className="font-mono text-xs break-all">{downloadResult.cid}</span>
            </Typography>
            <Typography variant="body2">
              Size: {(downloadResult.size / 1024 / 1024).toFixed(2)} MB
            </Typography>
            <Typography variant="body2">
              Duration: {downloadResult.duration_ms}ms
            </Typography>
            <Typography variant="body2">
              Verified: {downloadResult.verified ? 'Yes' : 'No'}
            </Typography>
            {downloadResult.filepath && (
              <Typography variant="body2">
                Saved to: {downloadResult.filepath}
              </Typography>
            )}
          </div>
        )}
        
        {error && (
          <div className="p-4 bg-red-50 border border-red-200 rounded-md">
            <Typography variant="body2" color="secondary">
              {error}
            </Typography>
          </div>
        )}
        
        <Button
          onClick={handleDownload}
          disabled={!cid || !!error || isDownloading}
          variant="filled"
          className="w-full"
        >
          {isDownloading ? 'Downloading...' : 'Download File'}
        </Button>
      </CardContent>
    </Card>
  );
}