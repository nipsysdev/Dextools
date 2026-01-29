import { useState } from "react";
import { useStore } from "@nanostores/react";
import { invoke } from "@tauri-apps/api/core";
import {
	Button,
	Dialog,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
	Input,
	Label,
} from "@nipsysdev/lsd-react";
import { $isAddPeerDialogOpened, $manualPeerConnections } from "../peersStore";

export default function AddPeerDialog() {
	const isDialogOpened = useStore($isAddPeerDialogOpened);
	const [peerId, setPeerId] = useState("");
	const [addresses, setAddresses] = useState<string[]>([""]);

	const isFormValid = peerId.trim() !== "" && addresses.some(addr => addr.trim() !== "");

	const handleAddAddress = () => {
		setAddresses([...addresses, ""]);
	};

	const handleRemoveAddress = (index: number) => {
		if (addresses.length > 1) {
			setAddresses(addresses.filter((_, i) => i !== index));
		}
	};

	const handleAddressChange = (index: number, value: string) => {
		const newAddresses = [...addresses];
		newAddresses[index] = value;
		setAddresses(newAddresses);
	};

	const handleConnect = async () => {
		try {
			const validAddresses = addresses.filter(addr => addr.trim() !== "");
			await invoke("connect_to_peer", {
				peerId: peerId.trim(),
				addresses: validAddresses,
			});

			// Update the peers store with the new connection
			$manualPeerConnections.set([
				...$manualPeerConnections.get(),
				{
					peerId: peerId.trim(),
					addresses: validAddresses,
				},
			]);

			// Reset form and close dialog
			setPeerId("");
			setAddresses([""]);
			$isAddPeerDialogOpened.set(false);
		} catch (error) {
			console.error("Failed to connect to peer:", error);
			// TODO: Show error message to user
		}
	};

	const handleClose = () => {
		setPeerId("");
		setAddresses([""]);
		$isAddPeerDialogOpened.set(false);
	};

	return (
		<Dialog
			open={isDialogOpened}
			onOpenChange={(open) => {
				if (!open) {
					handleClose();
				} else {
					$isAddPeerDialogOpened.set(open);
				}
			}}
		>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Add a new connection</DialogTitle>
				</DialogHeader>

				<div className="flex flex-col gap-y-10">
					<Input
						label="Peer ID"
						placeholder="Enter the ID of the peer"
						value={peerId}
						onChange={(e) => setPeerId(e.target.value)}
					/>

					<div>
						<div className="flex justify-between">
							<Label>Addresses</Label>{" "}
							<Button variant="outlined" size="sm" onClick={handleAddAddress}>
								Add address
							</Button>
						</div>

						{addresses.map((address, index) => (
							<div key={index} className="flex items-end gap-x-5 mt-5">
								<Input
									placeholder="Enter the address (e.g., /ip4/127.0.0.1/tcp/4001)"
									value={address}
									onChange={(e) => handleAddressChange(index, e.target.value)}
									className="flex-auto"
								/>
								{addresses.length > 1 && (
									<Button
										variant="outlined"
										size="sm"
										onClick={() => handleRemoveAddress(index)}
									>
										X
									</Button>
								)}
							</div>
						))}
					</div>
				</div>

				<DialogFooter className="mt-5">
					<Button
						variant="filled"
						onClick={handleConnect}
						disabled={!isFormValid}
					>
						Connect
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
