import type { CodegenConfig } from '@graphql-codegen/cli';

const config: CodegenConfig = {
	schema: 'http://localhost:4228',
	overwrite: true,
	documents: ['./src/**/*.svelte', './src/**/*.graphql', './src/**/*.ts'],
	generates: {
		'./src/gql/': {
			preset: 'client',
			plugins: ['typescript', 'typescript-operations', 'typescript-urql'],
			config: {
				useTypeImports: true,
				strictScalars: true,
				scalars: {
					Upload: 'File',
					NaiveDateTime: 'Date',
					Id: 'string',
					StripeProductId: 'string',
					InvoiceId: 'string',
					CustomerId: 'string',
					JSONObject: 'object'
				}
			}
		}
	}
};

export default config;
