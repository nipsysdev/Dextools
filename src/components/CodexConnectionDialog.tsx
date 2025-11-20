import { useStore } from '@nanostores/react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Progress,
  Typography,
} from '@nipsysdev/lsd-react';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import {
  $codexError,
  $codexPeerId,
  $codexStatus,
  $codexVersion,
  $isCodexDialogOpened,
  $nodeAddresses,
  CodexConnectionStatus,
} from '../stores/codexStore';

const connectToCodex = async () => {
  try {
    $codexStatus.set(CodexConnectionStatus.Connecting);
    $codexError.set(null);
    await invoke('connect_to_codex');
    // Status will be updated by the polling effect
  } catch (error) {
    console.error('Failed to connect to Codex:', error);
    $codexError.set(error as string);
    $codexStatus.set(CodexConnectionStatus.Error);
  }
};

export default function CodexConnectionDialog() {
  const isDialogOpened = useStore($isCodexDialogOpened);
  const codexStatus = useStore($codexStatus);
  const codexError = useStore($codexError);
  const codexPeerId = useStore($codexPeerId);
  const codexVersion = useStore($codexVersion);
  const nodeAddresses = useStore($nodeAddresses);

  const [peerId, setPeerId] = useState("");
  const [addresses, setAddresses] = useState([""]);
  const [showPeerConnect, setShowPeerConnect] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleConnectToPeer = async () => {
    try {
      await invoke("connect_to_peer", {
        peerId,
        addresses: addresses.filter((a) => a.trim()),
      });
      setShowPeerConnect(false);
      setPeerId("");
      setAddresses([""]);
      setError(null);
    } catch (error) {
      setError(`Failed to connect: ${error}`);
    }
  };

  const handleAddAddress = () => {
    setAddresses([...addresses, ""]);
  };

  const handleUpdateAddress = (index: number, value: string) => {
    const newAddresses = [...addresses];
    newAddresses[index] = value;
    setAddresses(newAddresses);
  };

  const handleRemoveAddress = (index: number) => {
    setAddresses(addresses.filter((_, i) => i !== index));
  };


  useEffect(() => {
    if (isDialogOpened) {
      // Only attempt to connect if currently disconnected
      // This prevents trying to create a new node when one already exists
      if (codexStatus === CodexConnectionStatus.Disconnected) {
        connectToCodex();
      }
    }
  }, [isDialogOpened, codexStatus]);

  useEffect(() => {
    // Update status immediately when dialog opens to ensure fresh data
    // The main App component handles the regular polling
    const updateStatusOnce = async () => {
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

    // Update status immediately when dialog opens
    if (isDialogOpened) {
      updateStatusOnce();
    }
  }, [isDialogOpened]);

  const disconnectFromCodex = async () => {
    try {
      await invoke('disconnect_from_codex');
      $codexStatus.set(CodexConnectionStatus.Disconnected);
      $codexError.set(null);
      $codexPeerId.set(null);
      $codexVersion.set(null);
    } catch (error) {
      console.error('Failed to disconnect from Codex:', error);
      $codexError.set(error as string);
    }
  };

  const getStatusDescription = (status: CodexConnectionStatus) => {
    switch (status) {
      case CodexConnectionStatus.Connecting:
        return 'Connecting to Codex network...';
      case CodexConnectionStatus.Connected:
        return 'Connected to Codex successfully';
      case CodexConnectionStatus.Error:
        return codexError || 'An error occurred while connecting to Codex';
      case CodexConnectionStatus.Disconnected:
        return 'Disconnected from Codex network';
      default:
        return '';
    }
  };


  return (
    <Dialog
      open={isDialogOpened}
      onOpenChange={(open) => {
        $isCodexDialogOpened.set(open);
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            Codex Status: <span className="capitalize">{codexStatus}</span>
          </DialogTitle>
          <DialogDescription>
            {getStatusDescription(codexStatus)}
          </DialogDescription>
        </DialogHeader>
        
        <Progress
          indeterminate={codexStatus === CodexConnectionStatus.Connecting}
          value={codexStatus === CodexConnectionStatus.Connected ? 100 : undefined}
        />

        {codexStatus === CodexConnectionStatus.Connected && (
          <div className="mt-4 space-y-2">
            {codexVersion && (
              <Typography variant="body2">
                Version: {codexVersion}
              </Typography>
            )}
            {codexPeerId && (
              <Typography variant="body2">
                Peer ID: {codexPeerId.slice(0, 20)}...
              </Typography>
            )}

            {/* Show user's node addresses */}
            {nodeAddresses.length > 0 && (
              <div className="mt-2">
                <Typography variant="body2" color="secondary">
                  Your Addresses (share these with others):
                </Typography>
                {nodeAddresses.map((addr, index) => (
                  <Typography
                    key={`addr-${index}-${addr.slice(0, 10)}`}
                    variant="body2"
                    className="font-mono text-xs break-all"
                  >
                    {addr}
                  </Typography>
                ))}
              </div>
            )}

            <button
              type="button"
              onClick={() => setShowPeerConnect(!showPeerConnect)}
              className="px-3 py-1 bg-lsd-surface-secondary hover:bg-lsd-surface-tertiary rounded-md text-sm transition-colors"
            >
              {showPeerConnect ? "Hide" : "Show"} Peer Connection
            </button>

            {showPeerConnect && (
              <div className="mt-4 space-y-4 p-4 bg-lsd-surface-secondary rounded-md">
                <Typography variant="h6">Connect to Peer</Typography>

                <div>
                  <div className="block text-sm font-medium mb-1">Peer ID:</div>
                  <input
                    type="text"
                    value={peerId}
                    onChange={(e) => setPeerId(e.target.value)}
                    placeholder="12D3KooW..."
                    className="w-full px-3 py-2 border border-lsd-border rounded-md bg-lsd-surface-primary"
                  />
                </div>

                <div>
                  <div className="block text-sm font-medium mb-1">Addresses:</div>
                  {addresses.map((addr, index) => (
                    <div key={`addr-${index}-${addr.slice(0, 8)}`} className="flex space-x-2 mb-2">
                      <input
                        type="text"
                        value={addr}
                        onChange={(e) => handleUpdateAddress(index, e.target.value)}
                        placeholder="/ip4/192.168.1.100/tcp/8080"
                        className="flex-1 px-3 py-2 border border-lsd-border rounded-md bg-lsd-surface-primary"
                      />
                      {addresses.length > 1 && (
                        <button
                          type="button"
                          onClick={() => handleRemoveAddress(index)}
                          className="px-2 py-1 bg-red-500 hover:bg-red-600 text-white rounded-md text-sm"
                        >
                          Remove
                        </button>
                      )}
                    </div>
                  ))}
                  <button
                    type="button"
                    onClick={handleAddAddress}
                    className="px-3 py-1 bg-lsd-surface-tertiary hover:bg-lsd-surface-secondary rounded-md text-sm transition-colors"
                  >
                    Add Address
                  </button>
                </div>

                {error && (
                  <Typography variant="body2" color="secondary">
                    {error}
                  </Typography>
                )}

                <div className="flex space-x-2">
                  <button
                    type="button"
                    onClick={handleConnectToPeer}
                    disabled={!peerId.trim() || !addresses.some((a) => a.trim())}
                    className="px-4 py-2 bg-lsd-primary hover:bg-lsd-primary-hover text-white rounded-md transition-colors disabled:opacity-50"
                  >
                    Connect to Peer
                  </button>
                </div>
              </div>
            )}
          </div>
        )}

        {codexStatus === CodexConnectionStatus.Error && (
          <div className="mt-4">
            <Typography variant="body2" color="secondary">
              {codexError}
            </Typography>
          </div>
        )}

        <div className="mt-6 flex justify-end space-x-2">
          {codexStatus === CodexConnectionStatus.Connected && (
            <button
              type="button"
              onClick={disconnectFromCodex}
              className="px-4 py-2 bg-lsd-surface-secondary hover:bg-lsd-surface-tertiary rounded-md transition-colors"
            >
              Disconnect
            </button>
          )}
          
          {(codexStatus === CodexConnectionStatus.Disconnected ||
            codexStatus === CodexConnectionStatus.Error) && (
            <button
              type="button"
              onClick={connectToCodex}
              className="px-4 py-2 bg-lsd-primary hover:bg-lsd-primary-hover text-white rounded-md transition-colors"
            >
              Connect
            </button>
          )}
          
          <button
            type="button"
            onClick={() => $isCodexDialogOpened.set(false)}
            className="px-4 py-2 bg-lsd-surface-secondary hover:bg-lsd-surface-tertiary rounded-md transition-colors"
          >
            Close
          </button>
        </div>
      </DialogContent>
    </Dialog>
  );
}