import { useStore } from "@nanostores/react";
import {
	Button,
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	Progress,
	Typography,
} from "@nipsysdev/lsd-react";
import { connectToStorage, disconnectFromStorage } from "../connectionService";
import {
	$connectionError,
	$connectionStatus,
	$isConnected,
	$isConnectionDialogOpened,
} from "../connectionStore";
import { ConnectionStatus } from "../connectionTypes";
import {
	getConnectionStatusDescription,
	getConnectionStatusText,
} from "../connectionUtils";

export default function ConnectionDialog() {
	const isDialogOpened = useStore($isConnectionDialogOpened);
	const connectionStatus = useStore($connectionStatus);
	const isConnected = useStore($isConnected);
	const connectionError = useStore($connectionError);

	return (
		<Dialog
			open={isDialogOpened}
			onOpenChange={(open) => {
				$isConnectionDialogOpened.set(open);
			}}
		>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>
						Status: {getConnectionStatusText(connectionStatus)}
					</DialogTitle>
					<DialogDescription>
						{getConnectionStatusDescription(connectionStatus, connectionError)}
					</DialogDescription>
				</DialogHeader>

				<Progress
					indeterminate={connectionStatus === ConnectionStatus.Connecting}
					value={isConnected ? 100 : undefined}
				/>

				{connectionStatus === ConnectionStatus.Error && (
					<div className="mt-4">
						<Typography variant="body2" color="secondary">
							{connectionError}
						</Typography>
					</div>
				)}

				<div className="mt-6 flex justify-end space-x-2">
					{![
						ConnectionStatus.Disconnected,
						ConnectionStatus.Initialized,
						ConnectionStatus.Error,
					].includes(connectionStatus) && (
						<Button variant="filled" onClick={disconnectFromStorage}>
							{isConnected ? "Disconnect" : "Abort"}
						</Button>
					)}

					{[
						ConnectionStatus.Disconnected,
						ConnectionStatus.Initialized,
						ConnectionStatus.Error,
					].includes(connectionStatus) && (
						<Button variant="filled" onClick={connectToStorage}>
							Connect
						</Button>
					)}
				</div>
			</DialogContent>
		</Dialog>
	);
}
