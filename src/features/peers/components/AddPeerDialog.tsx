import { useStore } from "@nanostores/react";
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
import { $isAddPeerDialogOpened } from "../peersStore";

export default function AddPeerDialog() {
	const isDialogOpened = useStore($isAddPeerDialogOpened);

	return (
		<Dialog
			open={isDialogOpened}
			onOpenChange={(open) => {
				$isAddPeerDialogOpened.set(open);
			}}
		>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Add a new connection</DialogTitle>
				</DialogHeader>

				<div className="flex flex-col gap-y-10">
					<Input label="Peer ID" placeholder="Enter the ID of the peer" />

					<div>
						<div className="flex justify-between">
							<Label>Addresses</Label>{" "}
							<Button variant="outlined" size="sm">
								Add address
							</Button>
						</div>

						<div className="flex items-end gap-x-5">
							<Input
								placeholder="Enter the ID of the peer"
								className="flex-auto"
							/>
						</div>

						<div className="flex items-end gap-x-5">
							<Input
								placeholder="Enter the ID of the peer"
								className="flex-auto"
							/>
							<Button variant="outlined" size="sm">
								X
							</Button>
						</div>
					</div>
				</div>

				<DialogFooter className="mt-5">
					<Button variant="filled">Connect</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
