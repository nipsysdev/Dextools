import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Progress, Typography } from '@nipsysdev/lsd-react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useState } from 'react';

interface UploadResult {
  cid: string;
  size: number;
  duration_ms: number;
  verified: boolean;
}

export default function UploadTab() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [uploadResult, setUploadResult] = useState<UploadResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedFilePath, setSelectedFilePath] = useState<string | null>(null);

  const handleFileSelect = async () => {
    try {
      // Use Tauri's file dialog to select a file and get its actual path
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'All Files',
          extensions: ['*']
        }]
      });

      if (selected && typeof selected === 'string') {
        setSelectedFilePath(selected);
        // Extract filename from path for display
        const fileName = selected.split(/[/\\]/).pop() || selected;
        setSelectedFile({
          name: fileName,
          size: 0, // We'll get this from the backend
        } as File);
        setUploadResult(null);
        setError(null);
      }
    } catch (err) {
      console.error('Failed to select file:', err);
      setError('Failed to select file');
    }
  };

  const handleUpload = async () => {
    if (!selectedFilePath) return;

    try {
      setIsUploading(true);
      setUploadProgress(0);
      setError(null);
      setUploadResult(null);

      console.log('Upload requested for:', selectedFilePath);
      
      // Simulate progress since we don't have real-time progress updates yet
      const progressInterval = setInterval(() => {
        setUploadProgress(prev => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return 90;
          }
          return prev + 10;
        });
      }, 200);

      try {
        const result = await invoke<UploadResult>('upload_file_to_codex', {
          filePath: selectedFilePath
        });
        
        clearInterval(progressInterval);
        setUploadProgress(100);
        setUploadResult(result);
      } catch (uploadErr) {
        clearInterval(progressInterval);
        throw uploadErr;
      }
    } catch (err) {
      console.error('Upload failed:', err);
      setError(`Upload failed: ${err}`);
    } finally {
      setIsUploading(false);
    }
  };

  return (
    <Card className="w-full max-w-md" style={{borderTop: 0}}>
      <CardHeader>
        <CardTitle>Upload File to Codex</CardTitle>
        <CardDescription>
          Select a file to upload to the Codex network
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div>
          <Button
            onClick={handleFileSelect}
            disabled={isUploading}
            variant="outlined"
            className="w-full"
          >
            Select File
          </Button>
        </div>
        
        {selectedFilePath && (
          <div className="p-4 bg-lsd-surface-secondary rounded">
            <Typography variant="body1">
              Selected: {selectedFile?.name || selectedFilePath}
            </Typography>
            <Typography variant="body2" color="secondary">
              Path: {selectedFilePath}
            </Typography>
          </div>
        )}
        
        {isUploading && (
          <div className="space-y-2">
            <Typography variant="body2">
              Uploading to Codex...
            </Typography>
            <Progress value={uploadProgress} />
          </div>
        )}
        
        {uploadResult && (
          <div className="space-y-2 p-4 bg-lsd-surface-secondary rounded-md">
            <Typography variant="body2" color="primary">
              Upload successful!
            </Typography>
            <Typography variant="body2">
              CID: <span className="font-mono text-xs break-all">{uploadResult.cid}</span>
            </Typography>
            <Typography variant="body2">
              Size: {(uploadResult.size / 1024 / 1024).toFixed(2)} MB
            </Typography>
            <Typography variant="body2">
              Duration: {uploadResult.duration_ms}ms
            </Typography>
            <Typography variant="body2">
              Verified: {uploadResult.verified ? 'Yes' : 'No'}
            </Typography>
          </div>
        )}
        
        {error && (
          <div className="p-4 bg-red-50 border border-red-200 rounded-md">
            <Typography variant="body2" color="secondary">
              Error: {error}
            </Typography>
          </div>
        )}
        
        <Button
          onClick={handleUpload}
          disabled={!selectedFilePath || isUploading}
          variant="filled"
          className="w-full"
        >
          {isUploading ? 'Uploading...' : 'Upload File'}
        </Button>
      </CardContent>
    </Card>
  );
}