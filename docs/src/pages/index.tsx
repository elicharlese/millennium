import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import { useColorMode } from '@docusaurus/theme-common';
// @ts-ignore
import { ColorModeProvider } from '@docusaurus/theme-common/internal';

function Homepage() {
	const { colorMode } = useColorMode();
	return <div id={`tailwind z-20 ${colorMode}`}>
		<div className='absolute z-0 top-0 inset-x-0 flex justify-center overflow-hidden pointer-events-none' style={{ userSelect: 'none' }}>
			<div className='w-[108rem] flex-none flex justify-end'>
				{colorMode === 'dark'
					? <img src={useBaseUrl('/img/tailwind-bg-dark.png')} alt='' className='w-[90rem] flex-none max-w-none' />
					: <img src={useBaseUrl('/img/tailwind-bg-light.png')} alt='' className='w-[71.75rem] flex-none max-w-none' />
				}
			</div>
		</div>
		<div className='lg:pt-32 py-16 overflow-hidden'>
			<div className='relative max-w-3xl mx-auto px-4 md:px-6 lg:px-8 lg:max-w-screen-xl'>
				<div className='max-w-screen-xl mx-auto px-4 md:px-6 lg:px-8'>
					<div className='max-w-4xl mx-auto text-center'>
						<h1 className='font-extrabold text-4xl sm:text-5xl lg:text-6xl tracking-tight text-center'>
							Build fast &amp; secure cross-platform web-based UIs
						</h1>
						<div className='py-16 flex flex-col items-center'>
							<div className='flex flex-row flex-wrap w-fit justify-center items-center'>
								<Link href='/docs/main/intro' className='w-fit bg-purple-500 hover:bg-purple-700 m-3 hover:text-purple-300 hover:no-underline shadow-xl text-white font-bold text-2xl py-4 px-8 rounded-full'>
									Learn more
								</Link>
								<Link href='/docs/main/your-first-app/prerequisites' className='w-fit bg-blue-500 hover:bg-blue-700 m-3 hover:text-blue-300 hover:no-underline shadow-xl text-white font-bold text-2xl py-4 px-8 rounded-full'>
									Get started
								</Link>
							</div>
							<pre className='w-fit my-8'>npm init millennium my-app</pre>
						</div>
					</div>
				</div>
			</div>
		</div>
		<div className='py-16 overflow-hidden diagonal-box'>
			<div className='diagonal-content max-w-2xl mx-auto px-4 md:px-6 lg:px-8 lg:max-w-screen-xl'>
				<div className='max-w-screen-xl pt-6 md:px-6 lg:px-8 flex flex-col'>
					<div className='max-w-4xl mx-auto text-center'>
						<h2 className='text-3xl leading-9 font-extrabold md:text-4xl md:leading-10'>Light as a feather</h2>
						<p className='mt-4 max-w-2xl text-xl leading-7 lg:mx-auto dark:text-gray-400'>
							Millennium utilizes the webview framework that already comes pre-installed with modern operating systems for ultra-lightweight binaries.
						</p>
						<p className='mt-4 max-w-2xl text-xl leading-7 lg:mx-auto dark:text-gray-400'>
							Millennium utilizes the webview framework that already comes pre-installed with modern operating systems for ultra-lightweight binaries.
						</p>
					</div>
					<div className='py-8 flex flex-col justify-items-center w-full mx-auto md:px-6 lg:max-w-screen-lg lg:px-8'>
						<img className='shadow-2xl rounded-2xl' src={useBaseUrl('/img/binary-chart.png')} />
						<p className='text-med text-center mt-4 dark:text-gray-500 light:text-gray-400'>
							Comparison of hello world binary size.
						</p>
					</div>
				</div>
			</div>
		</div>
	</div>
}

export default function Home(): JSX.Element {
	return (
		<Layout title='Home' description='Millennium is a cross-platform application framework written in Rust. With Millennium, you can create apps with a consistent UI that works across all desktop platforms with HTML, CSS, and JavaScript.'>
			<ColorModeProvider>
				<Homepage />
			</ColorModeProvider>
		</Layout>
	);
}
