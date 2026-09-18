import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TShirtIcon` instead.
 *
 * @example
 * ```svelte
 * <TShirt color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TShirt: Component<IconComponentProps, {}, "">;
type TShirt = ReturnType<typeof TShirt>;
export default TShirt;
