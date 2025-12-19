import { useStore } from "@nanostores/react";
import {
	Tabs,
	TabsContent,
	TabsList,
	TabsTrigger,
	Typography,
} from "@nipsysdev/lsd-react";
import ConnectionDialog from "./features/connection/components/ConnectionDialog";
import {
	$connectionStatus,
	$isConnected,
	$isConnectionDialogOpened,
} from "./features/connection/connectionStore";
import DownloadTab from "./features/download/components/DownloadTab";
import UploadTab from "./features/upload/components/UploadTab";
import "./App.css";
import { getConnectionStatusText } from "./features/connection/connectionUtils";
import NodeTab from "./features/node/components/NodeTab";
import AddPeerDialog from "./features/peers/components/AddPeerDialog";
import PeersTab from "./features/peers/components/PeersTab";

function App() {
	const connectionStatus = useStore($connectionStatus);
	const isConnected = useStore($isConnected);

	return (
		<div className="size-full flex flex-col bg-lsd-surface-primary pt-[env(safe-area-inset-top)] pb-[env(safe-area-inset-bottom)]">
			<header className="flex p-6 justify-between items-center">
				<Typography variant="h3">λ | storeman</Typography>
				<div className="flex items-center space-x-4">
					<Typography
						variant="subtitle1"
						color={isConnected ? "primary" : "secondary"}
						className="cursor-pointer font-bold hover:opacity-80"
						onClick={() => $isConnectionDialogOpened.set(true)}
					>
						{getConnectionStatusText(connectionStatus)}
					</Typography>
				</div>
			</header>

			<Tabs defaultValue="upload" className="flex-auto flex flex-col px-0.5">
				<TabsList fullWidth>
					<TabsTrigger value="upload" className="border-b-0">
						Upload
					</TabsTrigger>
					<TabsTrigger value="download" className="border-b-0">
						Download
					</TabsTrigger>
					<TabsTrigger value="node" className="border-b-0">
						Node
					</TabsTrigger>
					<TabsTrigger value="peers" className="border-b-0">
						Peers
					</TabsTrigger>
				</TabsList>

				<TabsContent value="upload" className="flex-auto mt-0 mb-0">
					<UploadTab />
				</TabsContent>

				<TabsContent value="download" className="flex-auto mt-0 mb-0">
					<DownloadTab />
				</TabsContent>

				<TabsContent value="node" className="flex-auto mt-0 mb-0">
					<NodeTab />
				</TabsContent>

				<TabsContent value="peers" className="flex-auto mt-0 mb-0">
					<PeersTab />
				</TabsContent>
			</Tabs>

			<ConnectionDialog />
			<AddPeerDialog />
		</div>
	);
}

export default App;
