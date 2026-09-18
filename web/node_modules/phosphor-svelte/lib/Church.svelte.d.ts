import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChurchIcon` instead.
 *
 * @example
 * ```svelte
 * <Church color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Church: Component<IconComponentProps, {}, "">;
type Church = ReturnType<typeof Church>;
export default Church;
