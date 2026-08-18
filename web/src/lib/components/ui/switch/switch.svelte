<script lang="ts">
	import { Switch as SwitchPrimitive } from "bits-ui";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		checked = $bindable(false),
		class: className,
		...restProps
	}: WithoutChildrenOrChild<SwitchPrimitive.RootProps> = $props();
</script>

<SwitchPrimitive.Root
	bind:ref
	data-slot="switch"
	class={cn(
		"border-input dark:bg-input/30 bg-input/60 data-[state=checked]:bg-primary data-[state=checked]:border-primary focus-visible:border-ring focus-visible:ring-ring/30 inline-flex h-4 w-7 shrink-0 items-center rounded-full border transition-colors focus-visible:ring-2 peer relative outline-none after:absolute after:-inset-x-2 after:-inset-y-2 disabled:cursor-not-allowed disabled:opacity-50",
		className
	)}
	bind:checked
	{...restProps}
>
	<SwitchPrimitive.Thumb
		data-slot="switch-thumb"
		class="bg-foreground data-[state=checked]:bg-primary-foreground pointer-events-none block size-3 rounded-full ring-0 transition-transform data-[state=checked]:translate-x-[0.8125rem] data-[state=unchecked]:translate-x-px"
	/>
</SwitchPrimitive.Root>
