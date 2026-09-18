import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShareNetworkIcon` instead.
 *
 * @example
 * ```svelte
 * <ShareNetwork color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShareNetwork: Component<IconComponentProps, {}, "">;
type ShareNetwork = ReturnType<typeof ShareNetwork>;
export default ShareNetwork;
