import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SynagogueIcon` instead.
 *
 * @example
 * ```svelte
 * <Synagogue color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Synagogue: Component<IconComponentProps, {}, "">;
type Synagogue = ReturnType<typeof Synagogue>;
export default Synagogue;
