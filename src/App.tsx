import { useStore } from '@nanostores/react';
import { Tabs, TabsContent, TabsList, TabsTrigger, Typography } from '@nipsysdev/lsd-react';
import { invoke } from '@tauri-apps/api/core';
import { useEffect } from 'react';
import CodexConnectionDialog from './components/CodexConnectionDialog';
import DownloadTab from './components/DownloadTab';
import UploadTab from './components/UploadTab';
import {
  $codexError,
  $codexPeerId,
  $codexStatus,
  $codexVersion,
  $isCodexDialogOpened,
  $nodeAddresses,
  CodexConnectionStatus,
} from './stores/codexStore';
import './App.css';

function App() {
  const codexStatus = useStore($codexStatus);
  const codexPeerId = useStore($codexPeerId);
  const codexVersion = useStore($codexVersion);
  const codexError = useStore($codexError);
  const isDialogOpened = useStore($isCodexDialogOpened);

  useEffect(() => {
    // Show connection dialog immediately on app load
    $isCodexDialogOpened.set(true);
    
    // Update status from backend on mount
    const updateStatus = async () => {
      try {
        const status = await invoke<CodexConnectionStatus>('get_codex_status');
        $codexStatus.set(status);
        
        const error = await invoke<string | null>('get_codex_error');
        $codexError.set(error);
        
        const peerId = await invoke<string | null>('get_codex_peer_id');
        $codexPeerId.set(peerId);
        
        const version = await invoke<string | null>('get_codex_version');
        $codexVersion.set(version);

        // Also update node addresses
        try {
          const addresses = await invoke<string[]>("get_node_addresses");
          $nodeAddresses.set(addresses);
        } catch (addrError) {
          // Don't fail the whole status update if addresses fail
          console.warn("Failed to get node addresses:", addrError);
        }
      } catch (error) {
        console.error('Failed to update Codex status:', error);
      }
    };

    updateStatus();
    const interval = setInterval(updateStatus, 2000);
    return () => clearInterval(interval);
  }, []);

  const getStatusText = () => {
    switch (codexStatus) {
      case CodexConnectionStatus.Connected:
        return 'Connected';
      case CodexConnectionStatus.Connecting:
        return 'Connecting...';
      case CodexConnectionStatus.Error:
        return 'Error';
      case CodexConnectionStatus.Disconnected:
      default:
        return 'Disconnected';
    }
  };

  const getStatusColor = () => {
    switch (codexStatus) {
      case CodexConnectionStatus.Connected:
        return 'primary';
      case CodexConnectionStatus.Connecting:
        return 'secondary';
      case CodexConnectionStatus.Error:
        return 'secondary';
      case CodexConnectionStatus.Disconnected:
      default:
        return 'secondary';
    }
  };

  const openConnectionDialog = () => {
    $isCodexDialogOpened.set(true);
  };

  return (
    <div className="min-h-screen bg-lsd-surface-primary">
      <header className="flex p-6 border-b border-lsd-border justify-between items-center">
        <Typography variant="h3">
          Dextools
        </Typography>
        <div className="flex items-center space-x-4">
          <Typography
            variant="subtitle1"
            color={getStatusColor() as any}
            className="cursor-pointer hover:opacity-80"
            onClick={openConnectionDialog}
            title={codexError || `Peer ID: ${codexPeerId || 'N/A'}\nVersion: ${codexVersion || 'N/A'}`}
          >
            {getStatusText()}
          </Typography>
          {codexStatus === CodexConnectionStatus.Disconnected && (
            <button
              type="button"
              onClick={openConnectionDialog}
              className="px-3 py-1 bg-lsd-primary hover:bg-lsd-primary-hover text-white rounded-md text-sm transition-colors"
            >
              Connect
            </button>
          )}
        </div>
      </header>
      
      <main className={`container mx-auto p-6 ${isDialogOpened ? 'blur-sm' : ''}`}>
        <Tabs defaultValue="upload" className="w-full max-w-md mx-auto">
          <TabsList fullWidth>
            <TabsTrigger value="upload" fullWidth>
              Upload
            </TabsTrigger>
            <TabsTrigger value="download" fullWidth>
              Download
            </TabsTrigger>
          </TabsList>
          
          <TabsContent value="upload" className="mt-6">
            <UploadTab />
          </TabsContent>
          
          <TabsContent value="download" className="mt-6">
            <DownloadTab />
          </TabsContent>
        </Tabs>
      </main>

      <CodexConnectionDialog />
    </div>
  );
}

export default App;
