import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NewspaperClippingIcon` instead.
 *
 * @example
 * ```svelte
 * <NewspaperClipping color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NewspaperClipping: Component<IconComponentProps, {}, "">;
type NewspaperClipping = ReturnType<typeof NewspaperClipping>;
export default NewspaperClipping;
