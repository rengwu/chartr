import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlaceholderIcon` instead.
 *
 * @example
 * ```svelte
 * <Placeholder color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Placeholder: Component<IconComponentProps, {}, "">;
type Placeholder = ReturnType<typeof Placeholder>;
export default Placeholder;
