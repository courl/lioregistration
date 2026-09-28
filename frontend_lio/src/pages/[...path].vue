<template>
  <UError
    :error="{
      status: 404,
      statusMessage: 'Page not found',
      message: 'The page you are looking for does not exist.',
    }"
  >
  </UError>
</template>

<route>
{
  meta: {
    layout: 'plain'
  }
}
</route>
