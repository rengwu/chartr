import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextColumnsIcon` instead.
 *
 * @example
 * ```svelte
 * <TextColumns color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextColumns: Component<IconComponentProps, {}, "">;
type TextColumns = ReturnType<typeof TextColumns>;
export default TextColumns;
