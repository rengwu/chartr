import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MosqueIcon` instead.
 *
 * @example
 * ```svelte
 * <Mosque color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Mosque: Component<IconComponentProps, {}, "">;
type Mosque = ReturnType<typeof Mosque>;
export default Mosque;
