<!--
  Completeness matrix for vue.{tags,complexity,cfg}.scm (vue.calls.scm is
  intentionally empty and vue.imports.scm does not exist — see the .scm
  header comments for why; JS/TS logic in <script> is out of scope,
  matching the confirmed Svelte precedent).

  One small, clearly commented construct per node-type variant found while
  cross-referencing the query files against arborium-vue 2.17.0's
  node-types.json (verified via `normalize syntax ast` / `normalize syntax
  query`, not node-types.json alone). A dedicated NEGATIVE section at the
  bottom covers near-miss constructs that must NOT match. This file must
  parse with zero (ERROR) nodes.
-->
<template>
  <!-- v-if/v-else-if/v-else on a `start_tag` element (has a body) -->
  <div v-if="a">a</div>
  <div v-else-if="b">b</div>
  <div v-else>c</div>

  <!-- v-for on a `start_tag` element -->
  <div v-for="item in items" :key="item.id">{{ item.id }}</div>

  <!-- v-if/v-else-if/v-else/v-for on a `self_closing_tag` element — a
       structurally distinct node type from start_tag (confirmed via
       node-types.json) that both vue.cfg.scm and vue.complexity.scm must
       match separately; components are conventionally self-closing, so
       this is the common shape, not an edge case. -->
  <MyComponent v-if="a" />
  <MyComponent v-else-if="b" />
  <MyComponent v-else />
  <MyComponent v-for="item in items" :key="item.id" />

  <!-- Non-branch/loop directives: must NOT be counted as @complexity or
       @cfg.branch/@cfg.loop, only as generic directive_attribute (which
       nothing here captures at all — no query models v-bind/v-on/v-model/
       v-slot/v-show as symbols, only as complexity-noise to exclude). -->
  <input v-model="text" />
  <span :title="tooltip">bind shorthand</span>
  <button @click="onClick">on shorthand</button>
  <div v-show="visible">show</div>
  <template v-slot:footer>
    <slot name="footer" />
  </template>
</template>

<script setup>
</script>

<style scoped>
</style>

<!--
  ===========================================================================
  NEGATIVE cases — must NOT match
  ===========================================================================
-->
<template>
  <!-- A directive whose name merely contains "if"/"for" as a substring
       must not satisfy the anchored `^v-(if|else-if|else|for)$` #match?
       pattern (guards against a future accidental prefix/substring match). -->
  <div data-if="not-a-directive">plain attribute named data-if, not v-if</div>

  <!-- A plain attribute (no `v-`/`:`/`@` prefix) is `attribute`, not
       `directive_attribute` — never a candidate for any directive query. -->
  <div class="plain-attribute">no directive at all</div>
</template>
