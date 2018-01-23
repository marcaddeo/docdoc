var gulp = require('gulp');
var sass = require('gulp-sass');
var minifyCSS = require('gulp-csso');

gulp.task('styles', function () {
    return gulp.src('src/scss/*.scss')
        .pipe(sass())
        .pipe(minifyCSS())
        .pipe(gulp.dest('assets/css'))
});

gulp.task('watch', function() {
  gulp.watch('src/scss/*.css', ['styles']);
});

gulp.task('default', [ 'styles' ]);
