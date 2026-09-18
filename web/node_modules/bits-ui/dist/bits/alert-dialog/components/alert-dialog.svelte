<script lang="ts">
	import { boxWith } from "svelte-toolbelt";
	import type { AlertDialogRootProps } from "../types.js";
	import { noop } from "../../../internal/noop.js";
	import { DialogRootState } from "../../dialog/dialog.svelte.js";

	let {
		open = $bindable(false),
		onOpenChange = noop,
		onOpenChangeComplete = noop,
		children,
	}: AlertDialogRootProps = $props();

	DialogRootState.create({
		variant: boxWith(() => "alert-dialog"),
		open: boxWith(
			() => open,
			(v) => {
				open = v;
				onOpenChange(v);
			}
		),
		onOpenChangeComplete: boxWith(() => onOpenChangeComplete),
	});
</script>

{@render children?.()}
