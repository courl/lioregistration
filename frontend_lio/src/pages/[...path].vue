<template>
  <UError
    :error="{
      status: 404,
      statusMessage: 'Page not found',
      message: 'The page you are looking for does not exist.',
    }"
    :clear="{ label: 'Back home', to: '/' }"
  />
</template>

<route>
{
  meta: {
    layout: 'plain'
  }
}
</route>
