import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TranslateIcon` instead.
 *
 * @example
 * ```svelte
 * <Translate color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Translate: Component<IconComponentProps, {}, "">;
type Translate = ReturnType<typeof Translate>;
export default Translate;
