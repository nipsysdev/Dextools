import { useStore } from "@nanostores/react";
import {
	Accordion,
	AccordionContent,
	AccordionItem,
	AccordionTrigger,
	Button,
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
	Typography,
} from "@nipsysdev/lsd-react";
import { $isAddPeerDialogOpened, $manualPeerConnections } from "../peersStore";

export default function PeersTab() {
	const peerConnections = useStore($manualPeerConnections);

	return (
		<Card className="size-full border-0!">
			<CardHeader className="border-x">
				<CardTitle>Connect to Peers</CardTitle>
				<CardDescription>Manage manual connections to peers</CardDescription>
			</CardHeader>
			<CardContent className="flex flex-col gap-y-6">
				<div className="text-right">
					<Button
						variant="outlined"
						onClick={() => $isAddPeerDialogOpened.set(true)}
					>
						New connection
					</Button>
				</div>

				{peerConnections.length ? (
					<Accordion type="single" collapsible className="w-full">
						<AccordionItem value="node1">
							<AccordionTrigger>Node 1</AccordionTrigger>
							<AccordionContent>Node 1 - Details</AccordionContent>
						</AccordionItem>
						<AccordionItem value="node2">
							<AccordionTrigger>Node 2</AccordionTrigger>
							<AccordionContent>Node 2 - Details</AccordionContent>
						</AccordionItem>
					</Accordion>
				) : (
					<Typography variant="body2" color="secondary" className="text-center">
						Nothing here
					</Typography>
				)}
			</CardContent>
		</Card>
	);
}
